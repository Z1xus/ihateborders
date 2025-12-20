use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPosition
{
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config
{
    #[serde(default)]
    pub auto_borderless_apps: Vec<String>,

    #[serde(default)]
    pub run_on_startup: bool,

    #[serde(default)]
    pub startup_admin: bool,

    #[serde(default)]
    pub window_position: Option<WindowPosition>,
}

impl Default for Config
{
    fn default() -> Self
    {
        Self {
            auto_borderless_apps: Vec::new(),
            run_on_startup: false,
            startup_admin: false,
            window_position: None,
        }
    }
}

impl Config
{
    pub fn load() -> Result<Self>
    {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()>
    {
        let config_path = Self::get_config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn get_config_path() -> Result<PathBuf>
    {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Failed to get executable directory"))?;
        Ok(exe_dir.join("ihateborders_config.json"))
    }

    pub fn add_auto_borderless(&mut self, process_name: String)
    {
        if !self.auto_borderless_apps.contains(&process_name) {
            self.auto_borderless_apps.push(process_name);
        }
    }

    pub fn remove_auto_borderless(&mut self, process_name: &str)
    {
        self.auto_borderless_apps.retain(|app| app != process_name);
    }

    pub fn is_auto_borderless(&self, process_name: &str) -> bool
    {
        self.auto_borderless_apps.contains(&process_name.to_string())
    }
}