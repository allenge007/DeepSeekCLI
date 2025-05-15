use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::error::Error;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub api_key: String,
}

pub fn read_config() -> Option<Config> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .ok()?;
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

pub fn set_config(api_key: &str) -> Result<(), Box<dyn Error>> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let config_path = PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("config.toml");

    // 创建配置目录（如果不存在）
    fs::create_dir_all(config_path.parent().unwrap())?;
    
    let config = Config {
        api_key: api_key.trim().to_string(),
    };
    let content = toml::to_string(&config)?;
    fs::write(&config_path, content)?;
    println!("配置文件已更新: {}", config_path.display());
    Ok(())
}