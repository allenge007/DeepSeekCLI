mod config;
mod history;
mod models;

//初始化i18n库
#[macro_use]
extern crate rust_i18n;
i18n!("locales");

use atty::Stream;
use clap::{Arg, Command, Subcommand};
use futures::StreamExt;
use reqwest::Client;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use config::{read_config, set_config, Config};
use history::*;
use models::{ChatMessage, ChatPayload, ResponseFormat, StreamingChunk};

#[derive(Subcommand)]
enum MemoryAction {
    /// 新对话（清空历史记录）
    New,
    /// 继续上一次对话
    Continue,
    // 无记忆模式
    NoMemory,
}

struct CliArgs {
    set_api: Option<String>,
    mem_action: Option<MemoryAction>,
    query: String,
    model: String,
    temperature: f32,
    no_memory: bool, // true 表示无记忆模式
}

fn parse_args() -> CliArgs {
    use rust_i18n::t;
    let matches = Command::new("ag")
        .about(t!("使用 DeepSeek API 进行多轮对话，并管理对话历史"))
        .arg(Arg::new("query").help(t!("查询内容")).index(1))
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .default_value("v3")
                .help(t!("模型版本, r1 表示 deepseek-reasoner")),
        )
        .arg(
            Arg::new("temperature")
                .short('t')
                .long("temperature")
                .default_value("1.0")
                .help(t!("温度（默认：1.0，范围：0.0-2.0，越高越随机）")),
        )
        // 当开启记忆模式时，仅允许 new 或 continue 子命令
        .subcommand(
            Command::new("new")
                .about(t!("新对话"))
                .arg(Arg::new("query").help(t!("查询内容")).index(1)),
        )
        .subcommand(
            Command::new("continue")
                .about(t!("继续上一次对话"))
                .arg(Arg::new("query").help(t!("查询内容")).index(1)),
        )
        .subcommand(
            Command::new("nomemory")
                .about(t!("无记忆模式"))
                .arg(Arg::new("query").help(t!("查询内容")).index(1)),
        )
        .subcommand(
            Command::new("set_api")
                .about(t!("设置 API Key"))
                .arg(Arg::new("api_key").help(t!("要设置的 API Key")).index(1)),
        )
        .get_matches();

    if let Some(sub_m) = matches.subcommand_matches("set_api") {
        let api_key = if let Some(key) = sub_m.get_one::<String>("api_key") {
            key.to_string()
        } else {
            print!("{}", t!("请输入 API Key:"));
            io::stdout().flush().unwrap();
            let mut key = String::new();
            io::stdin()
                .read_line(&mut key)
                .expect(t!("读取输入失败").as_ref());
            key.trim().to_string()
        };

        return CliArgs {
            set_api: Some(api_key),
            mem_action: None,
            query: "".to_string(),
            model: "".to_string(),
            temperature: 0.0,
            no_memory: false,
        };
    }

    let mem_action = if let Some(_) = matches.subcommand_matches("new") {
        Some(MemoryAction::New)
    } else if let Some(_) = matches.subcommand_matches("continue") {
        Some(MemoryAction::Continue)
    } else if let Some(_) = matches.subcommand_matches("nomemory") {
        Some(MemoryAction::NoMemory)
    } else {
        Some(MemoryAction::NoMemory)
    };

    // 获取查询内容：如果子命令中存在 query，则优先使用；否则使用全局参数
    let query = if let Some(sub_m) = matches.subcommand_matches("new") {
        sub_m
            .get_one::<String>("query")
            .unwrap_or_else(|| {
                eprintln!("{}", t!("请提供查询内容"));
                std::process::exit(1);
            })
            .to_string()
    } else if let Some(sub_m) = matches.subcommand_matches("continue") {
        sub_m
            .get_one::<String>("query")
            .unwrap_or_else(|| {
                eprintln!("{}", t!("请提供查询内容"));
                std::process::exit(1);
            })
            .to_string()
    } else if let Some(sub_m) = matches.subcommand_matches("nomemory") {
        sub_m
            .get_one::<String>("query")
            .unwrap_or_else(|| {
                eprintln!("{}", t!("请提供查询内容"));
                std::process::exit(1);
            })
            .to_string()
    } else if let Some(q) = matches.get_one::<String>("query") {
        q.to_string()
    } else {
        eprintln!("{}", t!("请提供查询内容"));
        std::process::exit(1);
    };

    let version = matches.get_one::<String>("version").unwrap();
    let model = if version == "r1" {
        "deepseek-reasoner".to_string()
    } else {
        "deepseek-chat".to_string()
    };

    let temperature = matches
        .get_one::<String>("temperature")
        .and_then(|t| t.parse::<f32>().ok())
        .unwrap_or(1.0);

    let no_memory = match mem_action {
        Some(MemoryAction::NoMemory) => true,
        _ => false,
    };

    CliArgs {
        set_api: None,
        mem_action,
        query,
        model,
        temperature,
        no_memory: no_memory,
    }
}

// 启动 spinner（仅在 stdout 为 tty 时有效）
fn start_spinner(model: &str) -> (Option<Arc<AtomicBool>>, Option<tokio::task::JoinHandle<()>>) {
    if atty::is(Stream::Stdout) {
        let sr = Arc::new(AtomicBool::new(true));
        let sr_clone = sr.clone();
        let model = model.to_string();
        let handle = tokio::spawn(async move {
            let spinner_chars = vec!["|", "/", "-", "\\"];
            let mut idx = 0;
            while sr_clone.load(Ordering::Relaxed) {
                eprint!(
                    "\r{}{}: {}",
                    model,
                    t!("加载中"),
                    spinner_chars[idx % spinner_chars.len()]
                );
                io::stderr().flush().unwrap();
                idx += 1;
                sleep(Duration::from_millis(100)).await;
            }
            eprint!("\r                   \r");
            io::stderr().flush().unwrap();
        });
        (Some(sr), Some(handle))
    } else {
        (None, None)
    }
}

/// 处理 SSE 流，实时输出 reasoning 及回答，返回最终回答内容
async fn process_stream(
    model: &str,
    mut stream: impl futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    spinner_running: Option<Arc<AtomicBool>>,
    mut spinner_handle: Option<tokio::task::JoinHandle<()>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut received_first_chunk = false;
    let mut thinking = true;
    let mut content = String::new();
    while let Some(item) = stream.next().await {
        let chunk = item?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            let line = line.trim();
            if line == "data: [DONE]" {
                return Ok(content);
            }
            if line.starts_with("data: ") {
                let data = line.trim_start_matches("data: ").trim();
                if !received_first_chunk {
                    if let Some(ref sr) = spinner_running {
                        sr.store(false, Ordering::Relaxed);
                    }
                    received_first_chunk = true;
                    if let Some(handle) = spinner_handle.take() {
                        handle.await?;
                    }
                    print!("\r{}{}:\n", model, t!("加载中"));
                }
                if let Ok(chunk_obj) = serde_json::from_str::<StreamingChunk>(data) {
                    if let Some(choice) = chunk_obj.choices.get(0) {
                        if let Some(delta) = &choice.delta {
                            // 输出思维链内容
                            if let Some(reasoning) = &delta.reasoning_content {
                                for c in reasoning.chars() {
                                    print!("{}", c);
                                    content.push(c);
                                    io::stdout().flush().unwrap();
                                }
                            }
                            // 输出最终回答
                            if let Some(delta_content) = &delta.content {
                                if model == "deepseek-reasoner" && thinking {
                                    thinking = false;
                                    println!();
                                    println!("\nanswer:\n");
                                }
                                for c in delta_content.chars() {
                                    print!("{}", c);
                                    content.push(c);
                                    io::stdout().flush().unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(content)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析主要参数
    let cli = parse_args();
    if let Some(api_key) = cli.set_api {
        return set_config(&api_key)
            .map_err(|e| format!("{},{}", t!("设置 API Key 失败: ").as_ref(), e).into());
    }

    let currnt_history_path = &current_history_path();

    // 读取管道传输的内容（如果有）
    let mut piped_input = String::new();
    if !atty::is(Stream::Stdin) {
        // 从标准输入读取管道内容
        use std::io::Read;
        io::stdin().read_to_string(&mut piped_input)?;
    }
    // 拼接管道内容与命令行查询内容
    let final_query = if piped_input.trim().is_empty() {
        cli.query.clone()
    } else {
        format!("{}\n{}", piped_input.trim(), cli.query)
    };

    // 根据记忆模式判断历史加载与保存
    let mut history_messages = if cli.no_memory {
        Vec::new()
    } else if let Some(MemoryAction::New) = cli.mem_action {
        Vec::new()
    } else {
        // 默认使用 continue 模式加载当前历史记录
        load_history(&currnt_history_path)
    };

    // 将用户提问加入对话历史
    history_messages.push(ChatMessage {
        role: "user".to_string(),
        content: final_query,
        reasoning_content: None,
        tool_calls: None,
    });

    let payload = ChatPayload {
        model: cli.model.clone(),
        messages: history_messages.clone(),
        frequency_penalty: 0,
        max_tokens: 2048,
        presence_penalty: 0,
        response_format: ResponseFormat {
            typ: "text".to_string(),
        },
        stop: None,
        stream: true,
        stream_options: Some(serde_json::json!({ "include_usage": true })),
        temperature: cli.temperature,
        top_p: 1.0,
        tools: None,
        tool_choice: "none".to_string(),
        logprobs: false,
        top_logprobs: None,
    };

    let cfg: Config = read_config().expect(
        t!("请检查配置文件 ~/.config/deepseek/config.toml 格式，或使用 set_api 重新设置 API Key")
            .as_ref(),
    );
    let api_key = cfg.api_key;
    let baseurl = "https://api.deepseek.com/chat/completions";
    let client = Client::new();
    let response = client
        .post(baseurl)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let err_text = response.text().await?;
        eprintln!("\x1b[31m{}{}\x1b[0m", t!("API 返回错误: "), err_text);
        std::process::exit(1);
    }

    let (spinner_running, spinner_handle) = start_spinner(&cli.model);
    let content = process_stream(
        &cli.model,
        response.bytes_stream(),
        spinner_running,
        spinner_handle,
    )
    .await?;

    println!();

    if !cli.no_memory {
        let mut new_history = payload.messages;
        new_history.push(ChatMessage {
            role: "assistant".to_string(),
            content: content.clone(),
            reasoning_content: None,
            tool_calls: None,
        });
        if let Some(MemoryAction::Continue) = cli.mem_action {
            delete_history(&currnt_history_path)?;
        }
        save_history(&new_history)?;
        // 绿色提示
        println!(
            "{}",
            t!("\x1b[32m历史记录已保存，使用 continue 自动继续上一次的对话。\x1b[0m")
        );
    } else {
        // 黄色提示
        println!("{}", t!("\x1b[33m无记忆模式下，不保存历史记录。\x1b[0m"));
    }

    Ok(())
}
