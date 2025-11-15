use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct KomfConfig {
    #[serde(default = "komf_url")]
    pub url: String,
}

fn komf_url() -> String {
    String::from("http://127.0.0.1:8085")
}

impl Default for KomfConfig {
    fn default() -> Self {
        Self {
            url: String::from("http://127.0.0.1:8085"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KomgaConfig {
    #[serde(default = "komga_url")]
    pub url: String,
    pub api_key: String,
    #[serde(default = "oneshots_dir")]
    pub oneshots_dir: String,
}

fn komga_url() -> String {
    String::from("http://127.0.0.1:25600")
}

fn oneshots_dir() -> String {
    String::from("_oneshots")
}

impl Default for KomgaConfig {
    fn default() -> Self {
        Self {
            url: String::from("http://127.0.0.1:25600"),
            api_key: String::new(),
            oneshots_dir: String::from("_oneshots"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub manga_dir: String,
    pub komga: KomgaConfig,
    #[serde(default)]
    pub komf: KomfConfig,
}

impl Default for Config {
    fn default() -> Self {
        let manga_dir = dirs::document_dir().map_or_else(
            || "./Mangas".to_string(),
            |p| format!("{}/Mangas", p.display()),
        );

        Self {
            manga_dir,
            komga: KomgaConfig::default(),
            komf: KomfConfig::default(),
        }
    }
}

impl Config {
    fn get_path() -> anyhow::Result<PathBuf> {
        let Some(config_dir) = dirs::config_dir() else {
            bail!("Failed to find config_directory")
        };

        let app_dir = format!("{}/{}", config_dir.display(), env!("CARGO_PKG_NAME"));

        if !Path::new(&app_dir).exists()
            && let Err(err) = fs::create_dir_all(&app_dir)
        {
            bail!("Failed to create config directory: {err}")
        }

        Ok(PathBuf::from(app_dir))
    }

    pub fn read() -> anyhow::Result<Self> {
        let path = Config::get_path()?.join("config.toml");
        if path.exists() {
            let config = toml::from_str(&fs::read_to_string(&path)?)?;
            Ok(config)
        } else {
            let config = Config::default();
            let config_str = toml::to_string_pretty(&config)?;
            fs::write(path, config_str)?;
            Ok(config)
        }
    }

    pub fn get_log_path() -> anyhow::Result<PathBuf> {
        Config::get_path().map(|v| v.join("cbz.log"))
    }
}
