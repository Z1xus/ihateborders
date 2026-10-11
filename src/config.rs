use std::path::{Path, PathBuf};
use std::{env, fs, io};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::system::Window;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Fit {
    pub resize: bool,
    pub display: Option<String>,
}

// the order is the priority when more than one rule fits a window
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Match {
    Title,
    Path,
    Name,
}

#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub path: PathBuf,
    pub title: String,
    #[serde(rename = "match")]
    pub by: Match,
    #[serde(flatten)]
    pub fit: Fit,
}

impl Rule {
    pub fn fits(&self, window: &Window) -> bool {
        let name = |path: &Path| path.file_name().map(|name| name.to_ascii_lowercase());
        let named = name(&self.path) == name(&window.path);
        match self.by {
            Match::Title => named && self.title == window.title,
            Match::Path => {
                let path = window.path.as_os_str();
                self.path.as_os_str().eq_ignore_ascii_case(path)
            }
            Match::Name => named,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)]
    pub fit: Fit,
    pub tray: bool,
    pub shortcut: String,
    pub apps: Vec<Rule>,
}

impl Default for Fit {
    fn default() -> Self {
        Self {
            resize: true,
            display: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fit: Fit::default(),
            tray: true,
            shortcut: String::new(),
            apps: Vec::new(),
        }
    }
}

fn file() -> PathBuf {
    let folder = PathBuf::from(env::var_os("APPDATA").unwrap_or_default());
    folder.join("ihateborders").join("config.toml")
}

pub fn load() -> Result<Config> {
    let file = file();
    match fs::read_to_string(&file) {
        Ok(text) => toml::from_str(&text).map_err(|error| {
            // the next save must not replace a file that the user wrote
            let _ = fs::rename(&file, file.with_extension("broken.toml"));
            error.into()
        }),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(error.into()),
    }
}

pub fn save(config: &Config) -> Result<()> {
    let file = file();
    fs::create_dir_all(file.with_file_name(""))?;
    Ok(fs::write(file, toml::to_string(config)?)?)
}
