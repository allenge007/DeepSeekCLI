use crate::models::ChatMessage;
use chrono::{DateTime, Local};
use serde_json;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 返回存放所有会话历史的目录
pub fn history_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("deepseek")
        .join("histories")
}

/// 创建一个新会话的路径（文件名使用当前时间戳）
pub fn new_history_path() -> PathBuf {
    let now: DateTime<Local> = Local::now();
    let filename = format!("{}.json", now.format("%Y%m%d%H%M%S"));
    history_dir().join(filename)
}

/// 列出所有历史会话文件（按名称排序）  
pub fn list_histories() -> io::Result<Vec<PathBuf>> {
    let dir = history_dir();
    fs::create_dir_all(&dir)?;
    let mut entries = fs::read_dir(&dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect::<Vec<_>>();
    entries.sort();
    Ok(entries)
}

/// 返回当前会话历史文件的路径；如果没有历史文件，则返回一个新生成的路径
pub fn current_history_path() -> PathBuf {
    match list_histories() {
        Ok(mut files) if !files.is_empty() => {
            files.sort();
            // 取最新（时间戳最大）的文件作为当前文件
            files.pop().unwrap()
        }
        _ => new_history_path(),
    }
}

/// 加载当前会话文件中的历史记录；如果文件不存在，则自动创建一个空文件并返回空列表
pub fn load_history(path: &Path) -> Vec<ChatMessage> {
    if !path.exists() {
        if let Err(e) = save_history_to_path(&path, &Vec::<ChatMessage>::new()) {
            eprintln!("创建历史文件失败: {}", e);
        }
        return Vec::new();
    }
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(history) = serde_json::from_str::<Vec<ChatMessage>>(&content) {
            return history;
        }
    }
    Vec::new()
}

/// 将历史记录保存到一个新的历史文件中（每次修改都会生成新文件）
pub fn save_history(messages: &[ChatMessage]) -> io::Result<()> {
    let path = new_history_path();
    save_history_to_path(&path, messages)
}

/// 将历史记录保存到指定路径的文件中
fn save_history_to_path<P: AsRef<Path>>(path: P, messages: &[ChatMessage]) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(messages)?;
    fs::write(path, json)
}

/// 删除指定的会话文件
pub fn delete_history<P: AsRef<Path>>(path: P) -> io::Result<()> {
    fs::remove_file(path)
}