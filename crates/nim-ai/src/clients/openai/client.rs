use std::time::Duration;

use reqwest::Client;

use crate::models::{ModelId, Provider};

use super::types::{ChatResponse, EmbeddingsResponse, Message};

/// An OpenAI-compatible client. The same schema is spoken by Azure OpenAI, Ollama, vLLM,
/// LM Studio, OpenRouter and Together, so `base_url` is all that differs between them.
pub struct OpenAI {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    model: ModelId,
}

impl OpenAI {
    pub fn new(model: ModelId, base_url: Option<String>, api_key: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: base_url.unwrap_or_else(|| Provider::OpenAI.base_url().to_string()),
            api_key: api_key.or_else(|| std::env::var(Provider::OpenAI.env()).ok()),
            model,
        }
    }

    /// A chat completion constrained to a JSON schema, so the model answers in a parseable shape
    /// rather than prose.
    pub async fn json<T: serde::de::DeserializeOwned>(
        &self,
        prompt: &str,
        input: &str,
        schema: serde_json::Value,
    ) -> nim_error::Result<T> {
        let body = serde_json::json!({
            "model": self.model.to_string(),
            "messages": [Message::system(prompt), Message::user(input)],
            "response_format": {
                "type": "json_schema",
                "json_schema": { "name": "result", "strict": true, "schema": schema },
            },
        });

        let response: ChatResponse = self.post("chat/completions", &body).await?;
        let content = response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| nim_error::Diagnostic::message("no completion returned"))?;

        Ok(serde_json::from_str(&content)?)
    }

    /// A plain prompt completion. Generation returns prose, so unlike `json` it is not constrained
    /// to a schema -- wrapping a summary in a JSON envelope only to unwrap it buys nothing.
    pub async fn prompt(&self, prompt: Option<&str>, input: &str, max_len: Option<usize>) -> nim_error::Result<String> {
        let mut messages = Vec::new();

        if let Some(prompt) = prompt {
            messages.push(Message::system(prompt));
        }

        messages.push(Message::user(input));

        let mut body = serde_json::json!({
            "model": self.model.to_string(),
            "messages": messages,
        });

        if let Some(max_len) = max_len {
            body["max_completion_tokens"] = serde_json::json!(max_len);
        }

        let response: ChatResponse = self.post("chat/completions", &body).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or_else(|| nim_error::Diagnostic::message("no completion returned"))
    }

    pub async fn complete(
        &self,
        messages: Vec<serde_json::Value>,
        tools: Vec<serde_json::Value>,
        max_len: Option<usize>,
    ) -> nim_error::Result<nim_chat::ChatAgentMessage> {
        let mut body = serde_json::json!({
            "model": self.model.to_string(),
            "messages": messages,
        });

        if !tools.is_empty() {
            body["tools"] = serde_json::Value::Array(tools);
        }

        if let Some(max_len) = max_len {
            body["max_completion_tokens"] = serde_json::json!(max_len);
        }

        let response: serde_json::Value = self.post("chat/completions", &body).await?;
        let message = response
            .get("choices")
            .and_then(serde_json::Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .ok_or_else(|| nim_error::Diagnostic::message("no completion returned"))?;
        let content = message
            .get("content")
            .and_then(serde_json::Value::as_str)
            .map(|value| nim_chat::ChatMessageContent::String(value.trim().to_string()));
        let reasoning = message
            .get("reasoning_content")
            .or_else(|| message.get("reasoning"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        let content = match (reasoning, content) {
            (Some(thinking), content) => {
                let mut parts = vec![nim_chat::ChatMessageContentPart::Thinking { thinking }];

                if let Some(nim_chat::ChatMessageContent::String(text)) = content
                    && !text.is_empty()
                {
                    parts.push(nim_chat::ChatMessageContentPart::Text { text });
                }

                Some(nim_chat::ChatMessageContent::Parts(parts))
            }
            (None, content) => content,
        };

        let tool_calls = message
            .get("tool_calls")
            .and_then(serde_json::Value::as_array)
            .map(|calls| {
                calls
                    .iter()
                    .map(|call| {
                        let function = call
                            .get("function")
                            .and_then(serde_json::Value::as_object)
                            .ok_or_else(|| nim_error::Diagnostic::message("tool call is missing function"))?;
                        let name = function
                            .get("name")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| nim_error::Diagnostic::message("tool call is missing name"))?
                            .to_string();
                        let args = function
                            .get("arguments")
                            .and_then(serde_json::Value::as_str)
                            .map(serde_json::from_str)
                            .transpose()?
                            .unwrap_or_default();
                        Ok(nim_chat::ToolCall {
                            id: call
                                .get("id")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or_default()
                                .to_string(),
                            name,
                            args,
                        })
                    })
                    .collect::<nim_error::Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(nim_chat::ChatAgentMessage {
            name: None,
            content,
            tool_calls,
            annotations: Vec::new(),
            extra: Default::default(),
        })
    }

    pub async fn embeddings(&self, text: &[&str]) -> nim_error::Result<Vec<Vec<f32>>> {
        let body = serde_json::json!({
            "model": self.model.to_string(),
            "input": text,
        });

        let response: EmbeddingsResponse = self.post("embeddings", &body).await?;
        let mut data = response.data;

        if data.len() != text.len() {
            return Err(nim_error::Diagnostic::message(format!(
                "expected {} embeddings, got {}",
                text.len(),
                data.len()
            )));
        }

        data.sort_by_key(|entry| entry.index);
        Ok(data.into_iter().map(|entry| entry.embedding).collect())
    }

    async fn post<T: serde::de::DeserializeOwned>(&self, path: &str, body: &serde_json::Value) -> nim_error::Result<T> {
        let url = format!("{}/{path}", self.base_url.trim_end_matches('/'));
        let mut request = self.client.post(&url).json(body);

        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await?;

        match response.error_for_status() {
            Err(err) => Err(err.into()),
            Ok(v) => Ok(v.json().await?),
        }
    }
}
