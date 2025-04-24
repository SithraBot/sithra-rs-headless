use std::path::Path;

use serde::Deserialize;
use tokio::fs;

const DEFAULT_CONFIG: &str = include_str!("../static/config.toml");

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(rename = "webdriver-url")]
    pub webdriver_url: String,
}

impl Config {
    pub async fn init(dir: impl AsRef<Path>) -> Self {
        let config_path = dir.as_ref().join("config.toml");
        let config = if config_path.exists() {
            let config_file = fs::read_to_string(config_path).await.unwrap();
            toml::from_str(&config_file).unwrap()
        } else {
            fs::create_dir_all(dir).await.unwrap();
            fs::write(config_path, DEFAULT_CONFIG).await.unwrap();
            toml::from_str(DEFAULT_CONFIG).unwrap()
        };
        config
    }
}
