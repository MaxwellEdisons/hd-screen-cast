use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
pub struct Display {
    pub id: Option<String>,
    pub name: String,
    pub device_name: String,
    pub hardware_id: String,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
    pub primary: bool,
    pub virtual_display: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub installed: bool,
    pub service: String,
    pub displays: Vec<Display>,
    pub target: String,
    pub virtual_installed: bool,
    pub virtual_enabled: bool,
    pub virtual_config: Option<VirtualConfig>,
    pub virtual_config_error: Option<String>,
    pub log: String,
    pub error: Option<String>,
}

impl Snapshot {
    pub fn running(&self) -> bool {
        self.service == "Running"
    }

    pub fn selected(&self, display: &Display) -> bool {
        if self.target.is_empty() {
            display.primary
        } else {
            display.id.as_ref() == Some(&self.target) || display.device_name == self.target
        }
    }

    pub fn target_display(&self) -> Option<&Display> {
        self.displays.iter().find(|display| self.selected(display))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Start,
    Stop,
    Restart,
    SelectDisplay { id: String },
    SetVirtualDisplay { enabled: bool },
    ConfigureVirtualDisplay { config: VirtualConfig },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualConfig {
    pub count: u32,
    pub modes: Vec<DisplayMode>,
}

impl VirtualConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=4).contains(&self.count) {
            return Err("虚拟屏数量需要在 1～4 块之间。".into());
        }
        if self.modes.is_empty() || self.modes.len() > 12 {
            return Err("请保留 1～12 个画面模式。".into());
        }
        for (index, mode) in self.modes.iter().enumerate() {
            if !(640..=7680).contains(&mode.width)
                || !(480..=4320).contains(&mode.height)
                || !(24..=240).contains(&mode.refresh_hz)
            {
                return Err(format!(
                    "模式 {} 超出范围：宽 640～7680，高 480～4320，刷新率 24～240Hz。",
                    index + 1
                ));
            }
            if self.modes[..index].contains(mode) {
                return Err(format!("模式 {} 与之前的模式重复。", index + 1));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AdminRequest {
    pub id: String,
    pub action: Action,
}

#[derive(Serialize, Deserialize)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Preferences {
    pub tv_address: String,
    pub adb_path: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LogDisplay {
    pub device_id: String,
    pub display_name: String,
    pub friendly_name: String,
    pub edid: Option<Edid>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Edid {
    pub manufacturer_id: String,
    pub product_code: String,
}

pub fn log_displays(log: &str) -> Vec<LogDisplay> {
    let Some((_, json)) = log.rsplit_once("Currently available display devices:") else {
        return Vec::new();
    };
    serde_json::Deserializer::from_str(json.trim_start())
        .into_iter::<Vec<LogDisplay>>()
        .next()
        .and_then(Result::ok)
        .unwrap_or_default()
}

pub fn configured_target(config: &str) -> String {
    config
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| key.trim() == "output_name")
        .map(|(_, value)| value.trim().to_owned())
        .next_back()
        .unwrap_or_default()
}

pub fn replace_target(config: &str, target: &str) -> String {
    let mut lines: Vec<&str> = config
        .lines()
        .filter(|line| {
            !line
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "output_name")
        })
        .collect();
    let setting = format!("output_name = {target}");
    lines.push(&setting);
    format!("{}\n", lines.join("\n"))
}
