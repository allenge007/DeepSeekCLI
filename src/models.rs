use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub reasoning_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub function: FunctionInfo,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub typ: String,
}

#[derive(Serialize)]
pub struct ChatPayload {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub frequency_penalty: i32,
    pub max_tokens: u32,
    pub presence_penalty: i32,
    pub response_format: ResponseFormat,
    pub stop: Option<Value>,
    pub stream: bool,
    pub stream_options: Option<Value>,
    pub temperature: f32,
    pub top_p: f32,
    pub tools: Option<Value>,
    pub tool_choice: String,
    pub logprobs: bool,
    pub top_logprobs: Option<Value>,
}

/// 以下结构体用于解析 SSE 流响应
#[derive(Deserialize)]
pub struct StreamingChunk {
    pub choices: Vec<StreamingChoice>,
}

#[derive(Deserialize)]
pub struct StreamingChoice {
    pub delta: Option<DeltaMessage>,
}

#[derive(Deserialize)]
pub struct DeltaMessage {
    pub reasoning_content: Option<String>,
    pub content: Option<String>,
}