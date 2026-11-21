#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub params: serde_json::Map<String, serde_json::Value>,
}

impl Tool {
    pub fn into_hf(self) -> Result<serde_json::Value, nim_error::Diagnostic> {
        Ok(serde_json::json!({
            "type": "function",
            "function": {
                "name": &self.name,
                "description": &self.description,
                "parameters": &self.params,
            }
        }))
    }

    pub fn into_openai(self) -> serde_json::Value {
        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.params,
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: serde_json::Map<String, serde_json::Value>,
}

impl ToolCall {
    pub fn into_hf(self) -> Result<serde_json::Value, nim_error::Diagnostic> {
        Ok(serde_json::json!({
            "id": &self.id,
            "type": "function",
            "function": {
                "name": &self.name,
                "arguments": &self.args,
            }
        }))
    }

    pub fn into_openai(self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "type": "function",
            "function": {
                "name": self.name,
                "arguments": serde_json::to_string(&self.args).unwrap_or_default(),
            }
        })
    }
}
