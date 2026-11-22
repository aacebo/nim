#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub params: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Map<String, serde_json::Value>,
}
