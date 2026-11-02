use crate::model::{
    self, Action, ActionResult, AdminRequest, Display, Preferences, Snapshot, VirtualConfig,
};
use serde::Deserialize;
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows::{
    Win32::{
        Foundation::CloseHandle,
        Graphics::Gdi::{
            DEVMODEW, DISPLAY_DEVICE_ACTIVE, DISPLAY_DEVICE_PRIMARY_DEVICE, DISPLAY_DEVICEW,
            ENUM_CURRENT_SETTINGS, EnumDisplayDevicesW, EnumDisplaySettingsW,
        },
        System::Threading::WaitForSingleObject,
        UI::{
            Shell::{
                IsUserAnAdmin, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
                ShellExecuteW,
            },
            WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL},
        },
    },
    core::{PCWSTR, w},
};

mod setup;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const DRIVER_QUERY: &str = r#"
$ErrorActionPreference='Stop'
$items=@(Get-PnpDevice -Class Display -ErrorAction Stop | Where-Object {
    $_.FriendlyName -eq 'Virtual Display Driver' -and
    @((Get-PnpDeviceProperty -InstanceId $_.InstanceId -KeyName 'DEVPKEY_Device_HardwareIds' -ErrorAction SilentlyContinue).Data) -contains 'Root\MttVDD'
})
"#;

fn sunshine_dir() -> PathBuf {
    std::env::var_os("ProgramW6432")
        .or_else(|| std::env::var_os("ProgramFiles"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:/Program Files"))
        .join("Sunshine")
}

fn config_path() -> PathBuf {
    sunshine_dir().join("config/sunshine.conf")
}

fn virtual_config_path() -> PathBuf {
    PathBuf::from("C:/VirtualDisplayDriver/vdd_settings.xml")
}

fn virtual_session_path() -> PathBuf {
    super::data_dir().join("virtual-session.json")
}

fn virtual_target(current: &Snapshot) -> bool {
    current
        .target_display()
        .is_some_and(|display| display.virtual_display)
        || model::log_displays(&read_log().unwrap_or_default())
            .iter()
            .any(|display| {
                (display.device_id == current.target || display.display_name == current.target)
                    && display
                        .edid
                        .as_ref()
                        .is_some_and(|edid| edid.manufacturer_id == "MTT")
            })
}

fn release_virtual(stop_service: bool) -> Result<String, String> {
    let current = snapshot();
    let owned = current.virtual_managed || virtual_target(&current);
    if (stop_service || owned) && current.running() {
        service("Stop")?;
    }
    // Keep a valid capture target for the next physical-screen session; do not resume capture here.
    if owned && current.installed {
        write_target("")?;
    }
    if current.virtual_enabled {
        set_virtual_driver(false)?;
        let start = Instant::now();
        while native_displays()
            .iter()
            .any(|display| display.virtual_display)
        {
            if start.elapsed() > Duration::from_secs(12) {
                return Err("虚拟驱动已请求关闭，但 Windows 尚未移除扩展屏。请刷新状态确认；本应用暂不退出。".into());
            }
            thread::sleep(Duration::from_millis(250));
        }
    }
    if virtual_session_path().exists() {
        fs::remove_file(virtual_session_path()).map_err(|error| error.to_string())?;
    }
    Ok("虚拟屏已释放，Windows 已移除对应扩展屏；不会自动改为共享主屏。".into())
}

fn start_virtual_stream() -> Result<String, String> {
    let current = snapshot();
    if !current.installed || !current.virtual_installed {
        return Err("请先完成 Sunshine 和虚拟屏驱动初始化。".into());
    }
    if let Some(error) = &current.error {
        return Err(error.clone());
    }
    if current.running() {
        service("Stop")?;
    }
    let result = (|| -> Result<(), String> {
        if !current.virtual_enabled {
            set_virtual_driver(true)?;
        }
        let start = Instant::now();
        while !native_displays()
            .iter()
            .any(|display| display.virtual_display)
        {
            if start.elapsed() > Duration::from_secs(15) {
                return Err("虚拟驱动启用后未发现可用扩展屏，请检查 Windows 显示设置。".into());
            }
            thread::sleep(Duration::from_millis(250));
        }
        fs::create_dir_all(super::data_dir()).map_err(|error| error.to_string())?;
        fs::write(virtual_session_path(), b"{\"managed\":true}")
            .map_err(|error| error.to_string())?;
        let start_time = SystemTime::now();
        service("Start")?;
        let start = Instant::now();
        let targets = loop {
            let fresh = fs::metadata(sunshine_dir().join("config/sunshine.log"))
                .and_then(|metadata| metadata.modified())
                .is_ok_and(|modified| modified >= start_time);
            let targets: Vec<_> = model::log_displays(&read_log().unwrap_or_default())
                .into_iter()
                .filter(|display| {
                    display
                        .edid
                        .as_ref()
                        .is_some_and(|edid| edid.manufacturer_id == "MTT")
                })
                .collect();
            if fresh && !targets.is_empty() {
                break targets;
            }
            if start.elapsed() > Duration::from_secs(20) {
                return Err("Sunshine 未能识别新启用的虚拟屏。".into());
            }
            thread::sleep(Duration::from_millis(250));
        };
        let target = targets
            .iter()
            .find(|display| display.device_id == current.target)
            .unwrap_or(&targets[0]);
        set_target(&target.device_id, true)?;
        Ok(())
    })();
    if let Err(error) = result {
        return match release_virtual(true) {
            Ok(_) => Err(format!("开启虚拟屏串流失败，已释放虚拟屏：{error}")),
            Err(cleanup) => Err(format!("开启失败：{error}\n清理未完成：{cleanup}")),
        };
    }
    Ok("虚拟屏串流已开启。请在电视端连接 Desktop；停止串流或退出应用会释放虚拟屏。".into())
}

fn set_virtual_driver(enabled: bool) -> Result<(), String> {
    let verb = if enabled {
        "Enable-PnpDevice"
    } else {
        "Disable-PnpDevice"
    };
    powershell(&format!("{DRIVER_QUERY}\nif($items.Count -eq 0){{throw 'Virtual display driver not found'}}; $items | ForEach-Object {{ {verb} -InstanceId $_.InstanceId -Confirm:$false -ErrorAction Stop }}")).map(|_| ())
}

fn write_virtual_config(config: &VirtualConfig) -> Result<PathBuf, String> {
    let path = virtual_config_path();
    let original =
        fs::read_to_string(&path).map_err(|error| format!("无法读取虚拟屏配置：{error}"))?;
    let updated = crate::virtual_config::replace(&original, config)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let backup = path.with_file_name(format!("vdd_settings.xml.backup-{stamp}"));
    fs::copy(&path, &backup).map_err(|error| format!("配置备份失败，未修改：{error}"))?;
    let temporary = path.with_file_name(format!("vdd_settings.xml.tmp-{stamp}"));
    fs::write(&temporary, updated).map_err(|error| error.to_string())?;
    fs::rename(temporary, path).map_err(|error| error.to_string())?;
    Ok(backup)
}

fn configure_virtual(config: VirtualConfig) -> Result<String, String> {
    config.validate()?;
    let current = snapshot();
    if let Some(error) = &current.error {
        return Err(error.clone());
    }
    if !current.virtual_installed {
        return Err("请先安装 MTT Virtual Display Driver。".into());
    }
    if let Some(error) = &current.virtual_config_error {
        return Err(error.clone());
    }
    if !current.virtual_enabled {
        let backup = write_virtual_config(&config)?;
        return Ok(format!(
            "已保存 {} 块虚拟屏及 {} 个可用画面模式，启用虚拟屏后生效。备份：{}",
            config.count,
            config.modes.len(),
            backup.display()
        ));
    }
    let selected_virtual = current
        .target_display()
        .is_some_and(|display| display.virtual_display);
    let old_virtual_ids: Vec<_> = current
        .displays
        .iter()
        .filter(|display| display.virtual_display)
        .filter_map(|display| display.id.as_ref())
        .collect();
    let old_index = old_virtual_ids
        .iter()
        .position(|id| **id == current.target)
        .unwrap_or(0);
    let original_sunshine = fs::read(config_path()).map_err(|error| error.to_string())?;
    // Save before interrupting the stream. Driver and Sunshine configuration are both recoverable.
    let backup = write_virtual_config(&config)?;
    let mut service_touched = false;
    let mut driver_touched = false;
    let application = (|| -> Result<(), String> {
        if current.running() {
            service_touched = true;
            service("Stop")?;
        }
        if selected_virtual {
            write_target("")?;
        }
        driver_touched = true;
        set_virtual_driver(false)?;
        thread::sleep(Duration::from_millis(500));
        set_virtual_driver(true)?;
        let started = Instant::now();
        loop {
            let count = native_displays()
                .iter()
                .filter(|display| display.virtual_display)
                .count();
            if count == config.count as usize {
                break;
            }
            if started.elapsed() > Duration::from_secs(20) {
                return Err(format!(
                    "驱动重启后仅检测到 {count} 块虚拟屏，预期 {} 块。",
                    config.count
                ));
            }
            thread::sleep(Duration::from_millis(500));
        }
        if current.running() {
            let start_time = SystemTime::now();
            service("Start")?;
            if selected_virtual {
                let started = Instant::now();
                let targets = loop {
                    let fresh = fs::metadata(sunshine_dir().join("config/sunshine.log"))
                        .and_then(|metadata| metadata.modified())
                        .is_ok_and(|modified| modified >= start_time);
                    let targets: Vec<_> = model::log_displays(&read_log().unwrap_or_default())
                        .into_iter()
                        .filter(|display| {
                            display
                                .edid
                                .as_ref()
                                .is_some_and(|edid| edid.manufacturer_id == "MTT")
                        })
                        .collect();
                    if fresh && targets.len() == config.count as usize {
                        break targets;
                    }
                    if started.elapsed() > Duration::from_secs(20) {
                        return Err("Sunshine 未能重新识别全部虚拟屏。".into());
                    }
                    thread::sleep(Duration::from_millis(500));
                };
                let target = targets
                    .iter()
                    .find(|display| display.device_id == current.target)
                    .or_else(|| targets.get(old_index))
                    .unwrap_or(&targets[0]);
                set_target(&target.device_id, true)?;
            }
        } else if selected_virtual {
            // A stopped service has no fresh display IDs; preserve a usable primary fallback.
            write_target("")?;
        }
        Ok(())
    })();
    if let Err(error) = application {
        let mut recovery_errors = Vec::new();
        if let Err(restore) = fs::copy(&backup, virtual_config_path()) {
            recovery_errors.push(format!("驱动配置恢复失败：{restore}"));
        }
        if let Err(restore) = fs::write(config_path(), original_sunshine) {
            recovery_errors.push(format!("共享目标恢复失败：{restore}"));
        }
        if driver_touched {
            if let Err(restore) = set_virtual_driver(false).and_then(|_| set_virtual_driver(true)) {
                recovery_errors.push(format!("虚拟驱动恢复失败：{restore}"));
            }
        }
        if service_touched && let Err(restore) = service("Restart") {
            recovery_errors.push(format!("服务恢复失败：{restore}"));
        }
        return Err(if recovery_errors.is_empty() {
            format!("配置应用失败，已恢复原配置：{error}")
        } else {
            format!(
                "配置应用失败：{error}\n{}\n驱动配置备份：{}",
                recovery_errors.join("\n"),
                backup.display()
            )
        });
    }
    Ok(format!(
        "已应用 {} 块虚拟屏及 {} 个可用画面模式。{}请在电视端重新连接；每块屏的当前模式可在 Windows 显示设置中选择。备份：{}",
        config.count,
        config.modes.len(),
        if selected_virtual && !current.running() {
            "Sunshine 原先已停止，共享目标暂时回退主屏。启动后重新选择虚拟屏。"
        } else {
            ""
        },
        backup.display()
    ))
}

fn read_log() -> Result<String, String> {
    let path = sunshine_dir().join("config/sunshine.log");
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    // Preserve startup identity and recent diagnostics without loading an unbounded log.
    let length = file.metadata().map_err(|error| error.to_string())?.len();
    let mut data = Vec::new();
    if length > 2 * 1024 * 1024 {
        file.by_ref()
            .take(1024 * 1024)
            .read_to_end(&mut data)
            .map_err(|error| error.to_string())?;
        data.extend_from_slice(b"\n[... earlier diagnostics omitted ...]\n");
        file.seek(SeekFrom::End(-1024 * 1024))
            .map_err(|error| error.to_string())?;
    }
    file.read_to_end(&mut data)
        .map_err(|error| error.to_string())?;
    Ok(String::from_utf8_lossy(&data).into_owned())
}

fn run(mut command: Command, timeout: Duration) -> Result<String, String> {
    command
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let mut stdout = child.stdout.take().ok_or("无法读取命令输出")?;
    let mut stderr = child.stderr.take().ok_or("无法读取命令错误")?;
    let out_reader = thread::spawn(move || {
        let mut data = Vec::new();
        let _ = stdout.read_to_end(&mut data);
        data
    });
    let err_reader = thread::spawn(move || {
        let mut data = Vec::new();
        let _ = stderr.read_to_end(&mut data);
        data
    });
    let started = Instant::now();
    let status = loop {
        match child.try_wait().map_err(|error| error.to_string())? {
            Some(status) => break status,
            None if started.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("操作超时，请检查服务或电视连接后重试。".into());
            }
            None => thread::sleep(Duration::from_millis(100)),
        }
    };
    let out = String::from_utf8_lossy(&out_reader.join().unwrap_or_default())
        .trim()
        .to_owned();
    let err = String::from_utf8_lossy(&err_reader.join().unwrap_or_default())
        .trim()
        .to_owned();
    if status.success() {
        Ok(out)
    } else {
        Err(if err.is_empty() { out } else { err })
    }
}

fn powershell(script: &str) -> Result<String, String> {
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &format!("[Console]::OutputEncoding=[Text.UTF8Encoding]::new(); {script}"),
    ]);
    run(command, Duration::from_secs(50))
}

fn text(buffer: &[u16]) -> String {
    String::from_utf16_lossy(
        &buffer[..buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(buffer.len())],
    )
}

fn native_displays() -> Vec<Display> {
    let mut displays = Vec::new();
    for index in 0..64 {
        let mut device = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        if !unsafe { EnumDisplayDevicesW(PCWSTR::null(), index, &mut device, 0) }.as_bool() {
            break;
        }
        if !device.StateFlags.contains(DISPLAY_DEVICE_ACTIVE) {
            continue;
        }
        let mut mode = DEVMODEW {
            dmSize: size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };
        if !unsafe {
            EnumDisplaySettingsW(
                PCWSTR(device.DeviceName.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut mode,
            )
        }
        .as_bool()
        {
            continue;
        }
        let mut monitor = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        let _ =
            unsafe { EnumDisplayDevicesW(PCWSTR(device.DeviceName.as_ptr()), 0, &mut monitor, 0) };
        displays.push(Display {
            name: text(&monitor.DeviceString),
            device_name: text(&device.DeviceName),
            hardware_id: text(&monitor.DeviceID),
            width: mode.dmPelsWidth,
            height: mode.dmPelsHeight,
            refresh_hz: mode.dmDisplayFrequency,
            primary: device.StateFlags.contains(DISPLAY_DEVICE_PRIMARY_DEVICE),
            virtual_display: text(&monitor.DeviceID).to_uppercase().contains("MTT1337"),
            ..Default::default()
        });
    }
    displays
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SystemStatus {
    service: String,
    virtual_installed: bool,
    virtual_enabled: bool,
}

pub fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot {
        installed: sunshine_dir().join("sunshine.exe").is_file(),
        displays: native_displays(),
        ..Default::default()
    };
    snapshot.packages_available = setup::available();
    snapshot.virtual_managed = virtual_session_path().exists();
    match fs::read_to_string(virtual_config_path())
        .map_err(|error| error.to_string())
        .and_then(|xml| crate::virtual_config::parse(&xml))
    {
        Ok(config) => snapshot.virtual_config = Some(config),
        Err(error) => snapshot.virtual_config_error = Some(format!("无法读取虚拟屏配置：{error}")),
    }
    let config = fs::read_to_string(config_path());
    match config {
        Ok(config) => {
            snapshot.target = model::configured_target(config.trim_start_matches('\u{feff}'))
        }
        Err(error) if snapshot.installed => {
            snapshot.error = Some(format!("无法读取 Sunshine 配置：{error}"))
        }
        _ => {}
    }
    match read_log() {
        Ok(log) => {
            let known_displays = model::log_displays(&log);
            for display in &mut snapshot.displays {
                if let Some(known) = known_displays.iter().find(|known| {
                    known.display_name == display.device_name
                        && known.edid.as_ref().is_some_and(|edid| {
                            display.hardware_id.to_uppercase().contains(
                                &format!("{}{}", edid.manufacturer_id, edid.product_code)
                                    .to_uppercase(),
                            )
                        })
                }) {
                    display.id = Some(known.device_id.clone());
                    display.name.clone_from(&known.friendly_name);
                    display.virtual_display |= known
                        .edid
                        .as_ref()
                        .is_some_and(|edid| edid.manufacturer_id == "MTT");
                }
            }
            snapshot.log = log
                .lines()
                .rev()
                .take(90)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");
        }
        Err(error) if snapshot.installed => {
            snapshot.error = Some(format!("无法读取 Sunshine 日志：{error}"))
        }
        _ => {}
    }
    let script = format!(
        "{DRIVER_QUERY}\n$s=Get-Service -Name SunshineService -ErrorAction SilentlyContinue; [pscustomobject]@{{Service=$(if($s){{$s.Status.ToString()}}else{{'Missing'}}); VirtualInstalled=($items.Count -gt 0); VirtualEnabled=(@($items | Where-Object Status -eq 'OK').Count -gt 0)}} | ConvertTo-Json -Compress"
    );
    match powershell(&script).and_then(|json| {
        serde_json::from_str::<SystemStatus>(&json).map_err(|error| error.to_string())
    }) {
        Ok(status) => {
            snapshot.service = status.service;
            snapshot.virtual_installed = status.virtual_installed;
            snapshot.virtual_enabled = status.virtual_enabled;
        }
        Err(error) => {
            snapshot.service = "Unknown".into();
            snapshot.error = Some(format!("读取系统状态失败：{error}"));
        }
    }
    snapshot
}

fn service(operation: &str) -> Result<(), String> {
    let command = match operation {
        "Start" => "Start-Service",
        "Stop" => "Stop-Service",
        "Restart" => "Restart-Service",
        _ => return Err("不支持的服务操作".into()),
    };
    powershell(&format!(
        "$ErrorActionPreference='Stop'; {command} -Name SunshineService -ErrorAction Stop"
    ))
    .map(|_| ())
}

fn write_target(target: &str) -> Result<PathBuf, String> {
    let path = config_path();
    let config = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    let backup = path.with_file_name(format!("sunshine.conf.backup-{timestamp}"));
    fs::copy(&path, &backup).map_err(|error| format!("备份失败，未修改配置：{error}"))?;
    let temporary = path.with_file_name(format!("sunshine.conf.tmp-{timestamp}"));
    fs::write(
        &temporary,
        model::replace_target(config.trim_start_matches('\u{feff}'), target),
    )
    .map_err(|error| error.to_string())?;
    fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    Ok(backup)
}

fn set_target(target: &str, running: bool) -> Result<String, String> {
    let backup = write_target(target)?;
    if running && let Err(error) = service("Restart") {
        let restored = fs::copy(&backup, config_path());
        if let Err(restore_error) = restored {
            return Err(format!(
                "切换失败：{error}；配置恢复失败：{restore_error}。备份：{}",
                backup.display()
            ));
        }
        let _ = service("Start");
        return Err(format!("切换失败，已恢复原配置：{error}"));
    }
    Ok(format!(
        "共享屏幕已更新。{}备份：{}",
        if running {
            "请在电视端重新连接。"
        } else {
            "启动 Sunshine 后生效。"
        },
        backup.display()
    ))
}

fn perform(action: Action) -> Result<String, String> {
    if !unsafe { IsUserAnAdmin() }.as_bool() {
        return Err("此操作需要管理员权限".into());
    }
    match action {
        Action::Initialize {
            sunshine,
            virtual_driver,
        } => setup::initialize(sunshine, virtual_driver),
        Action::Start => {
            if virtual_target(&snapshot()) {
                return start_virtual_stream();
            }
            service("Start")?;
            Ok("Sunshine 已启动。".into())
        }
        Action::Stop => release_virtual(true),
        Action::StartVirtualStream => start_virtual_stream(),
        Action::ReleaseVirtualSession => release_virtual(false),
        Action::Restart => {
            service("Restart")?;
            Ok("Sunshine 已重启，请在电视端重新连接。".into())
        }
        Action::SelectDisplay { id } => {
            let current = snapshot();
            if current.error.is_some() {
                return Err(current.error.unwrap());
            }
            if !id.is_empty()
                && !current
                    .displays
                    .iter()
                    .any(|display| display.id.as_ref() == Some(&id))
            {
                return Err("目标屏幕未连接，或显示器列表已变化。请刷新后重试。".into());
            }
            let target_is_virtual = current
                .displays
                .iter()
                .any(|display| display.id.as_ref() == Some(&id) && display.virtual_display);
            if target_is_virtual {
                write_target(&id)?;
                return start_virtual_stream();
            }
            let result = set_target(&id, current.running())?;
            if current.virtual_managed {
                if current.virtual_enabled {
                    set_virtual_driver(false)?;
                }
                if virtual_session_path().exists() {
                    fs::remove_file(virtual_session_path()).map_err(|error| error.to_string())?;
                }
            }
            Ok(result)
        }
        Action::SetVirtualDisplay { enabled } => {
            if enabled {
                start_virtual_stream()
            } else {
                release_virtual(false)
            }
        }
        Action::ConfigureVirtualDisplay { config } => configure_virtual(config),
    }
}

fn encode(data: &[u8]) -> String {
    data.iter().map(|value| format!("{value:02x}")).collect()
}

fn decode(data: &str) -> Result<Vec<u8>, String> {
    if data.len() > 4096 || !data.len().is_multiple_of(2) || !data.is_ascii() {
        return Err("无效的管理请求".into());
    }
    (0..data.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&data[index..index + 2], 16).map_err(|_| "无效的管理请求".into())
        })
        .collect()
}

fn result_path(id: &str) -> Result<PathBuf, String> {
    if id.is_empty() || id.len() > 40 || !id.bytes().all(|value| value.is_ascii_digit()) {
        return Err("无效的请求编号".into());
    }
    let path = super::data_dir().join("operations");
    fs::create_dir_all(&path).map_err(|error| error.to_string())?;
    Ok(path.join(format!("{id}.json")))
}

pub fn admin_entry(encoded: &str) -> Result<(), String> {
    let request: AdminRequest =
        serde_json::from_slice(&decode(encoded)?).map_err(|error| error.to_string())?;
    let path = result_path(&request.id)?;
    let result = match perform(request.action) {
        Ok(message) => ActionResult { ok: true, message },
        Err(message) => ActionResult { ok: false, message },
    };
    fs::write(
        path,
        serde_json::to_vec(&result).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

pub fn execute(action: Action) -> Result<String, String> {
    if unsafe { IsUserAnAdmin() }.as_bool() {
        return perform(action);
    }
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos()
        .to_string();
    let path = result_path(&id)?;
    let request = encode(
        &serde_json::to_vec(&AdminRequest { id, action }).map_err(|error| error.to_string())?,
    );
    let executable: Vec<u16> = std::env::current_exe()
        .map_err(|error| error.to_string())?
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let parameters: Vec<u16> = format!("--admin {request}")
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(executable.as_ptr()),
        lpParameters: PCWSTR(parameters.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    unsafe { ShellExecuteExW(&mut info) }
        .map_err(|error| format!("未获得管理员授权，未执行操作：{error}"))?;
    if info.hProcess.is_invalid() {
        return Err("无法获取管理进程状态，请刷新确认结果。".into());
    }
    let wait = unsafe { WaitForSingleObject(info.hProcess, 300_000) };
    let _ = unsafe { CloseHandle(info.hProcess) };
    if wait != windows::Win32::Foundation::WAIT_OBJECT_0 {
        return Err("管理操作尚未完成，请稍后刷新确认状态，避免重复操作。".into());
    }
    let result: ActionResult = serde_json::from_slice(
        &fs::read(&path).map_err(|error| format!("无法读取操作结果：{error}"))?,
    )
    .map_err(|error| error.to_string())?;
    let _ = fs::remove_file(path);
    if result.ok {
        Ok(result.message)
    } else {
        Err(result.message)
    }
}

pub fn open(target: &str) -> Result<(), String> {
    let target: Vec<u16> = target.encode_utf16().chain(Some(0)).collect();
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(target.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize <= 32 {
        Err("无法打开系统页面。".into())
    } else {
        Ok(())
    }
}

pub fn television_metrics(preferences: &Preferences) -> Result<String, String> {
    let address = preferences.tv_address.trim();
    let valid = address.parse::<std::net::IpAddr>().is_ok()
        || address.parse::<std::net::SocketAddr>().is_ok();
    if !valid {
        return Err("请输入电视 IP 地址，可包含端口，例如 192.168.1.100:5555。".into());
    }
    let endpoint = match address.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(ip)) => format!("{ip}:5555"),
        Ok(std::net::IpAddr::V6(ip)) => format!("[{ip}]:5555"),
        Err(_) => address.to_owned(),
    };
    let adb = if preferences.adb_path.trim().is_empty() {
        "adb"
    } else {
        preferences.adb_path.trim()
    };
    let invoke = |args: &[&str]| {
        let mut command = Command::new(adb);
        command.args(args);
        run(command, Duration::from_secs(18))
    };
    let connected = invoke(&["connect", &endpoint])?;
    if connected.contains("failed") || connected.contains("unable") {
        return Err(connected);
    }
    let dump = invoke(&[
        "-s",
        &endpoint,
        "shell",
        "uiautomator",
        "dump",
        "/sdcard/sunshine-control-ui.xml",
    ])?;
    if dump.contains("ERROR") {
        return Err("无法读取电视界面。请进入 Moonlight 串流并开启性能信息。".into());
    }
    let xml = invoke(&[
        "-s",
        &endpoint,
        "shell",
        "cat",
        "/sdcard/sunshine-control-ui.xml",
    ])?;
    let overlay = xml
        .split("<node ")
        .find(|node| node.contains("com.limelight:id/performanceOverlay"))
        .ok_or("未发现性能信息。请在 Moonlight 高级设置中开启性能信息，并保持串流画面。")?;
    let value = overlay
        .split_once("text=\"")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map(|(text, _)| text)
        .ok_or("电视性能信息格式异常")?;
    let value = value
        .replace("&#10;", "\n")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    let logs = invoke(&[
        "-s",
        &endpoint,
        "logcat",
        "-d",
        "-t",
        "600",
        "com.limelight.LimeLog:I",
        "moonlight-common-c:I",
        "*:S",
    ])?;
    let events: Vec<_> = logs
        .lines()
        .filter(|line| {
            line.contains("queue overflow") || line.contains("Dequeue input buffer ran long")
        })
        .collect();
    Ok(format!(
        "{value}\n\n最近日志中的队列/阻塞事件：{}\n{}\n\n此信息是当前采样；日志事件可能来自之前的串流会话。",
        events.len(),
        events
            .into_iter()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .join("\n")
    ))
}
