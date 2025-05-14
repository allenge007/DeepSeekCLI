mod config;
mod history;
mod models;

use atty::Stream;
use clap::{Arg, Command, ArgAction, Subcommand};
use futures::StreamExt;
use reqwest::Client;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};

use config::{read_config, Config};
use history::*;
use models::{ChatMessage, ChatPayload, ResponseFormat, StreamingChunk};

#[derive(Subcommand)]
enum MemoryAction {
    /// 新对话（清空历史记录）
    New,
    /// 继续上一次对话
    Continue,
}

struct CliArgs {
    mem_action: Option<MemoryAction>,
    query: String,
    model: String,
    temperature: f32,
    no_memory: bool,   // true 表示无记忆模式
}

fn parse_args() -> CliArgs {
    let matches = Command::new("ag")
        .version("1.0")
        .about("使用 DeepSeek API 进行多轮对话，并管理对话历史")
        .arg(Arg::new("query").help("查询内容").index(1))
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .default_value("v3")
                .help("模型版本, r1 表示 deepseek-reasoner"),
        )
        .arg(
            Arg::new("temperature")
                .short('t')
                .long("temperature")
                .default_value("1")
                .help("温度（默认：1）"),
        )
        // 使用 --memory 表示记忆模式，否则为无记忆模式
        .arg(
            Arg::new("memory")
                .long("memory")
                .short('m')
                .help("记忆模式：启用后每次调用 API 时保存历史记录")
                .action(ArgAction::SetTrue),
        )
        // 当开启记忆模式时，仅允许 new 或 continue 子命令
        .subcommand(
            Command::new("new")
                .about("新对话")
                .arg(Arg::new("query").help("查询内容").index(1)),
        )
        .subcommand(
            Command::new("continue")
                .about("继续上一次对话")
                .arg(Arg::new("query").help("查询内容").index(1)),
        )
        .get_matches();

    // 如果在记忆模式下且存在子命令，则优先从子命令中获取查询内容
    let query = if let Some(sub_m) = matches.subcommand_matches("new") {
        sub_m.get_one::<String>("query").unwrap_or_else(|| {
            eprintln!("请提供查询内容");
            std::process::exit(1);
        }).to_string()
    } else if let Some(sub_m) = matches.subcommand_matches("continue") {
        sub_m.get_one::<String>("query").unwrap_or_else(|| {
            eprintln!("请提供查询内容");
            std::process::exit(1);
        }).to_string()
    } else if let Some(q) = matches.get_one::<String>("query") {
        q.to_string()
    } else {
        eprintln!("请提供查询内容");
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

    let memory = matches.get_flag("memory");
    // 当记忆模式开启时，允许 new 或 continue 作为子命令；否则均为无记忆模式
    let mem_action = if memory {
        if let Some(_) = matches.subcommand_matches("new") {
            Some(MemoryAction::New)
        } else {
            // 未指定默认为 continue
            Some(MemoryAction::Continue)
        }
    } else {
        None
    };

    CliArgs {
        mem_action,
        query,
        model,
        temperature,
        no_memory: !memory,
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
                eprint!("\r{}🤖: {}", model, spinner_chars[idx % spinner_chars.len()]);
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
                    print!("\r{}🤖:\n", model);
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
                                    sleep(Duration::from_millis(10)).await;
                                }
                                println!("\n");
                            }
                            // 输出最终回答
                            if let Some(delta_content) = &delta.content {
                                if model == "deepseek-reasoner" && thinking {
                                    thinking = false;
                                    println!("\nanswer:\n");
                                }
                                for c in delta_content.chars() {
                                    print!("{}", c);
                                    content.push(c);
                                    io::stdout().flush().unwrap();
                                    sleep(Duration::from_millis(10)).await;
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
        response_format: ResponseFormat { typ: "text".to_string() },
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

    let cfg: Config = read_config()
        .expect("请检查配置文件 ~/.config/deepseek/config.toml 格式");
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
        eprintln!("\x1b[31mAPI 返回错误: {}\x1b[0m", err_text);
        std::process::exit(1);
    }

    let (spinner_running, spinner_handle) = start_spinner(&cli.model);
    let content = process_stream(
        &cli.model,
        response.bytes_stream(),
        spinner_running,
        spinner_handle,
    ).await?;

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
        println!("\x1b[32m历史记录已保存，下一轮对话自动继续上一次的对话。\x1b[0m");
    } else {
        // 黄色提示
        println!("\x1b[33m无记忆模式下，不保存历史记录。\x1b[0m");
    }

    Ok(())
}