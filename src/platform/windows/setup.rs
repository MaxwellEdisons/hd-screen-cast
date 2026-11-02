use super::*;
use sha2::{Digest, Sha256};
use windows::{Win32::Devices::DeviceAndDriverInstallation::*, core::GUID};

#[derive(Deserialize)]
struct Manifest {
    files: Vec<PackageFile>,
}
#[derive(Deserialize)]
struct PackageFile {
    path: String,
    sha256: String,
}

fn package_root() -> Result<PathBuf, String> {
    std::env::current_exe()
        .map_err(|error| error.to_string())?
        .parent()
        .map(PathBuf::from)
        .ok_or("无法确定应用目录。".into())
}

pub(super) fn available() -> bool {
    package_root().is_ok_and(|root| {
        root.join("packages/Sunshine-Windows-AMD64-installer.msi")
            .is_file()
            && root
                .join("packages/virtual-display-driver/MttVDD.inf")
                .is_file()
            && root.join("packages/vc_redist.x64.exe").is_file()
    })
}

fn verify() -> Result<PathBuf, String> {
    let manifest: Manifest = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/packages-manifest.json"
    )))
    .map_err(|error| error.to_string())?;
    let root = package_root()?;
    for package in manifest.files {
        let path = root.join(&package.path);
        let mut file = fs::File::open(&path).map_err(|error| {
            format!(
                "安装组件缺失：{}（{error}）。请完整解压离线安装包。",
                package.path
            )
        })?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        if format!("{:x}", hash.finalize()) != package.sha256 {
            return Err(format!(
                "安装组件校验失败：{}。未运行安装程序，请重新下载完整离线包。",
                package.path
            ));
        }
    }
    Ok(root)
}

fn installer(mut command: Command) -> Result<bool, String> {
    command
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            return match status.code() {
                Some(0 | 1638) => Ok(false),
                Some(3010 | 1641) => Ok(true),
                code => Err(format!(
                    "组件安装失败，退出代码 {code:?}。已完成的组件会保留，可修复后重新初始化；安装日志位于应用数据目录。"
                )),
            };
        }
        if start.elapsed() > Duration::from_secs(420) {
            return Err(
                "安装程序仍在运行，请完成 Windows 安装窗口后刷新状态，避免重复初始化。".into(),
            );
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn install_driver(inf: &std::path::Path) -> Result<bool, String> {
    let inf: Vec<u16> = inf
        .as_os_str()
        .to_string_lossy()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let hardware: Vec<u16> = "Root\\MttVDD".encode_utf16().chain([0, 0]).collect();
    let mut class = GUID::zeroed();
    let mut name = [0u16; 128];
    unsafe { SetupDiGetINFClassW(PCWSTR(inf.as_ptr()), &mut class, &mut name, None) }
        .map_err(|error| error.to_string())?;
    let list = unsafe { SetupDiCreateDeviceInfoList(Some(&class), None) }
        .map_err(|error| error.to_string())?;
    let mut device = SP_DEVINFO_DATA {
        cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
        ..Default::default()
    };
    let mut registered = false;
    let result = (|| -> windows::core::Result<bool> {
        unsafe {
            SetupDiCreateDeviceInfoW(
                list,
                PCWSTR(name.as_ptr()),
                &class,
                w!("Virtual Display Driver"),
                None,
                DICD_GENERATE_ID,
                Some(&mut device),
            )?;
            let hardware_bytes: Vec<u8> = hardware
                .iter()
                .flat_map(|unit| unit.to_le_bytes())
                .collect();
            SetupDiSetDeviceRegistryPropertyW(
                list,
                &mut device,
                SPDRP_HARDWAREID,
                Some(&hardware_bytes),
            )?;
            SetupDiCallClassInstaller(DIF_REGISTERDEVICE, list, Some(&device))?;
            registered = true;
            let mut reboot = windows::core::BOOL::default();
            UpdateDriverForPlugAndPlayDevicesW(
                None,
                w!("Root\\MttVDD"),
                PCWSTR(inf.as_ptr()),
                INSTALLFLAG_FORCE,
                Some(&mut reboot),
            )?;
            Ok(reboot.as_bool())
        }
    })();
    if result.is_err() && registered {
        let removed = unsafe { SetupDiRemoveDevice(list, &mut device) };
        if !removed.as_bool() {
            let _ = unsafe { SetupDiDestroyDeviceInfoList(list) };
            return Err("驱动安装失败，且临时设备清理未完成。请在设备管理器中检查 Virtual Display Driver 后重试。".into());
        }
    }
    let _ = unsafe { SetupDiDestroyDeviceInfoList(list) };
    result.map_err(|error| format!("虚拟驱动安装失败：{error}。如果 Windows 显示发布者确认窗口，请自行确认；应用不会导入根证书或关闭系统签名检查。"))
}

pub(super) fn initialize(want_sunshine: bool, want_driver: bool) -> Result<String, String> {
    if !want_sunshine && !want_driver {
        return Err("请至少选择一个需要安装的组件。".into());
    }
    let root = verify()?;
    let current = snapshot();
    if current.service == "Unknown" {
        return Err("无法读取系统状态，未安装组件。请刷新状态后重试。".into());
    }
    let install_sunshine = want_sunshine && !current.installed;
    let install_virtual = want_driver && !current.virtual_installed;
    if !install_sunshine && !install_virtual {
        return Ok("所选组件已经安装，没有重复安装或覆盖现有配置。".into());
    }
    fs::create_dir_all(super::super::data_dir()).map_err(|error| error.to_string())?;
    let mut runtime = Command::new(root.join("packages/vc_redist.x64.exe"));
    runtime.args(["/install", "/passive", "/norestart"]);
    let mut reboot = installer(runtime)?;
    if install_sunshine {
        let mut msi = Command::new("msiexec.exe");
        msi.arg("/i")
            .arg(root.join("packages/Sunshine-Windows-AMD64-installer.msi"))
            .args(["/passive", "/norestart", "/L*v"])
            .arg(super::super::data_dir().join("sunshine-install.log"));
        reboot |= installer(msi)?;
    }
    if install_virtual {
        let destination = PathBuf::from("C:/VirtualDisplayDriver");
        fs::create_dir_all(&destination).map_err(|error| error.to_string())?;
        for file in ["MttVDD.inf", "mttvdd.cat", "MttVDD.dll"] {
            fs::copy(
                root.join("packages/virtual-display-driver").join(file),
                destination.join(file),
            )
            .map_err(|error| error.to_string())?;
        }
        let settings = destination.join("vdd_settings.xml");
        if !settings.exists() {
            fs::write(&settings, r#"<?xml version="1.0" encoding="utf-8"?>
<vdd_settings><monitors><count>1</count></monitors><gpu><friendlyname>default</friendlyname></gpu><resolutions><resolution><width>2560</width><height>1440</height><refresh_rate>60</refresh_rate></resolution></resolutions><options><CustomEdid>false</CustomEdid><PreventSpoof>false</PreventSpoof><EdidCeaOverride>false</EdidCeaOverride><HardwareCursor>true</HardwareCursor><SDR10bit>false</SDR10bit><HDRPlus>false</HDRPlus><logging>false</logging><debuglogging>false</debuglogging></options></vdd_settings>
"#).map_err(|error| error.to_string())?;
        }
        reboot |= install_driver(&destination.join("MttVDD.inf"))?;
    }
    let after = snapshot();
    if install_sunshine && !after.installed {
        return Err("安装程序已结束，但尚未发现 Sunshine。请检查安装日志并刷新状态。".into());
    }
    if install_virtual && !after.virtual_installed {
        return Err("驱动安装程序已结束，但尚未发现虚拟驱动。请重启电脑并刷新状态。".into());
    }
    Ok(format!(
        "初始化完成，现有配置与配对信息保持不变。{}下一步：打开 Sunshine 管理页面设置账号，配对电视，再选择共享屏幕。",
        if reboot {
            "Windows 提示需要重启电脑，请先手动重启。"
        } else {
            ""
        }
    ))
}
