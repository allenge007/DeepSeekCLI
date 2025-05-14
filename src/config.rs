use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub api_key: String,
}

pub fn read_config() -> Option<Config> {
    let home = env::var("HOME").ok()?;
    let config_path = PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("config.toml");
    if !config_path.exists() {
        if let Err(e) = fs::create_dir_all(config_path.parent().unwrap()) {
            eprintln!("创建配置目录失败: {}", e);
        }
        return None;
    }
    let content = fs::read_to_string(config_path).ok()?;
    toml::from_str(&content).ok()
}