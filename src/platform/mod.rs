use crate::model::{Action, Preferences, Snapshot};
use std::path::PathBuf;

#[cfg(windows)]
mod windows;

pub fn data_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("SunshineControl")
}

pub fn load_preferences() -> Preferences {
    std::fs::read(data_dir().join("preferences.json"))
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_default()
}

pub fn save_preferences(preferences: &Preferences) -> Result<(), String> {
    std::fs::create_dir_all(data_dir()).map_err(|error| error.to_string())?;
    std::fs::write(
        data_dir().join("preferences.json"),
        serde_json::to_vec_pretty(preferences).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

pub fn snapshot() -> Snapshot {
    #[cfg(windows)]
    {
        windows::snapshot()
    }
    #[cfg(not(windows))]
    {
        Snapshot {
            error: Some("此版本仅支持 Windows。".into()),
            ..Default::default()
        }
    }
}

pub fn execute(action: Action) -> Result<String, String> {
    #[cfg(windows)]
    {
        windows::execute(action)
    }
    #[cfg(not(windows))]
    {
        let _ = action;
        Err("此版本仅支持 Windows。".into())
    }
}

pub fn admin_entry(encoded: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        windows::admin_entry(encoded)
    }
    #[cfg(not(windows))]
    {
        let _ = encoded;
        Err("此版本仅支持 Windows。".into())
    }
}

pub fn open(target: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        windows::open(target)
    }
    #[cfg(not(windows))]
    {
        let _ = target;
        Err("此版本仅支持 Windows。".into())
    }
}

pub fn television_metrics(preferences: &Preferences) -> Result<String, String> {
    #[cfg(windows)]
    {
        windows::television_metrics(preferences)
    }
    #[cfg(not(windows))]
    {
        let _ = preferences;
        Err("此版本仅支持 Windows。".into())
    }
}
