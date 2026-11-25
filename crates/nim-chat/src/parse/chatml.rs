use crate::{ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, Parse, ParseMode, Parser, ToolCall};

pub fn parse_message(p: &mut Parser<'_>) -> nim_error::Result<ChatMessage> {
    Ok(parse_agent_message(p)?.into())
}

pub fn parse_agent_message(p: &mut Parser<'_>) -> nim_error::Result<ChatAgentMessage> {
    let mode = *p.mode();
    let src = p.remaining();
    p.advance_by(src.len());
    let mut src = src.trim();
    let mut thinking = None;

    if let Some(rest) = src.strip_prefix("<think>") {
        match rest.split_once("</think>") {
            Some((value, remaining)) => {
                thinking = Some(value.trim().to_string());
                src = remaining;
            }
            None if mode == ParseMode::Strict => {
                return Err(nim_error::Diagnostic::message("unterminated Qwen reasoning block"));
            }
            None => src = rest,
        }
    }

    let anchor = Parser::from_src(src, mode);
    let mut text = String::new();
    let mut tool_calls = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while index < src.len() {
        if let Some(end) = anchor.seek_code_end(index) {
            index = end;
            continue;
        }

        let Some(remaining) = src[index..].strip_prefix("<tool_call>") else {
            index += src[index..].chars().next().unwrap().len_utf8();
            continue;
        };

        text.push_str(&src[start..index]);

        let Some((raw, rest)) = remaining.split_once("</tool_call>") else {
            if mode == ParseMode::Strict {
                return Err(nim_error::Diagnostic::message("unterminated Qwen tool call"));
            }

            text.push_str(&src[index..]);
            start = src.len();
            break;
        };

        let mut parser = Parser::from_src(raw.trim(), mode);

        match parse_tool_call(&mut parser) {
            Ok(mut call) => {
                if call.id.is_empty() {
                    call.id = format!("call_{}", tool_calls.len());
                }

                tool_calls.push(call);
            }
            Err(error) if mode == ParseMode::Strict => return Err(error),
            Err(_) => {
                text.push_str("<tool_call>");
                text.push_str(raw);
                text.push_str("</tool_call>");
            }
        }

        index = src.len() - rest.len();
        start = index;
    }

    text.push_str(&src[start..]);
    let text = text.trim();
    let content = if text.is_empty() {
        None
    } else {
        Some(ChatMessageContent::parse(&mut Parser::from_src(text, mode))?)
    };

    let content = match (thinking, content) {
        (Some(thinking), content) => {
            let mut output = ChatMessageContent::Parts(vec![ChatMessageContentPart::Thinking { thinking }]);

            match content {
                Some(ChatMessageContent::String(text)) => output.push(ChatMessageContentPart::Text { text }),
                Some(ChatMessageContent::Parts(parts)) => {
                    for part in parts {
                        output.push(part);
                    }
                }
                None => {}
            }

            Some(output)
        }
        (None, content) => content,
    };

    Ok(ChatAgentMessage {
        name: None,
        content,
        tool_calls,
        annotations: Vec::new(),
        extra: Default::default(),
    })
}

pub fn parse_tool_call(p: &mut Parser<'_>) -> nim_error::Result<ToolCall> {
    let src = p.remaining();
    p.advance_by(src.len());
    let value: serde_json::Value = serde_json::from_str(src.trim())?;
    let object = value
        .as_object()
        .ok_or_else(|| nim_error::Diagnostic::message("Qwen tool call must be an object"))?;
    let name = object
        .get("name")
        .or_else(|| object.get("function").and_then(|v| v.get("name")))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| nim_error::Diagnostic::message("Qwen tool call is missing a name"))?
        .to_string();
    let args = object
        .get("arguments")
        .or_else(|| object.get("args"))
        .or_else(|| object.get("function").and_then(|v| v.get("arguments")))
        .map(|value| match value {
            serde_json::Value::String(value) => serde_json::from_str(value),
            serde_json::Value::Object(value) => Ok(value.clone()),
            _ => Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Qwen tool arguments must be an object",
            ))),
        })
        .transpose()?
        .unwrap_or_default();
    let id = object
        .get("id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .unwrap_or_default();

    Ok(ToolCall { id, name, args })
}
