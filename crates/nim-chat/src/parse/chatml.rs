use crate::{ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ParseMode, Parser, ToolCall};

pub fn parse_message(p: &mut Parser<'_>) -> nim_error::Result<ChatMessage> {
    Ok(parse_agent_message(p)?.into())
}

pub fn parse_agent_message(p: &mut Parser<'_>) -> nim_error::Result<ChatAgentMessage> {
    let mode = *p.mode();
    let source = p.remaining();
    p.advance_by(source.len());
    let mut source = source.trim();
    let mut thinking = None;

    if let Some(rest) = source.strip_prefix("<think>") {
        match rest.split_once("</think>") {
            Some((value, remaining)) => {
                thinking = Some(value.trim().to_string());
                source = remaining;
            }
            None if mode == ParseMode::Strict => {
                return Err(nim_error::Diagnostic::message("unterminated Qwen reasoning block"));
            }
            None => source = rest,
        }
    }

    let mut text = String::new();
    let mut tool_calls = Vec::new();

    while let Some((before, remaining)) = source.split_once("<tool_call>") {
        text.push_str(before);

        let Some((raw, rest)) = remaining.split_once("</tool_call>") else {
            if mode == ParseMode::Strict {
                return Err(nim_error::Diagnostic::message("unterminated Qwen tool call"));
            }

            text.push_str("<tool_call>");
            text.push_str(remaining);
            source = "";
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

        source = rest;
    }

    text.push_str(source);
    let text = text.trim();
    let content = if text.is_empty() {
        None
    } else {
        Some(parse_message_content(&mut Parser::from_src(text, mode))?)
    };

    let content = match (thinking, content) {
        (Some(thinking), content) => {
            let mut parts = vec![ChatMessageContentPart::Thinking { thinking }];

            match content {
                Some(ChatMessageContent::String(text)) => parts.push(ChatMessageContentPart::Text { text }),
                Some(ChatMessageContent::Parts(values)) => parts.extend(values),
                _ => {}
            }

            Some(ChatMessageContent::Parts(parts))
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
    let source = p.remaining();
    p.advance_by(source.len());
    let value: serde_json::Value = serde_json::from_str(source.trim())?;
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

pub fn parse_message_content(p: &mut Parser<'_>) -> nim_error::Result<ChatMessageContent> {
    let mut parts = Vec::new();

    while !p.is_eof() {
        parts.push(parse_message_content_part(p)?);
    }

    if parts.is_empty() {
        return Err(nim_error::Diagnostic::message("message content is empty"));
    }

    if parts.len() == 1
        && let ChatMessageContentPart::Text { text } = &parts[0]
    {
        return Ok(ChatMessageContent::String(text.clone()));
    }

    Ok(ChatMessageContent::Parts(parts))
}

pub fn parse_message_content_part(p: &mut Parser<'_>) -> nim_error::Result<ChatMessageContentPart> {
    let source = p.remaining();

    if let Some(rest) = source.strip_prefix("<think>") {
        if let Some((thinking, remaining)) = rest.split_once("</think>") {
            p.advance_by(source.len() - remaining.len());
            return Ok(ChatMessageContentPart::Thinking {
                thinking: thinking.trim().to_string(),
            });
        }

        if *p.mode() == ParseMode::Strict {
            return Err(nim_error::Diagnostic::message("unterminated Qwen reasoning block"));
        }
    }

    if let Some((len, url)) = parse_url(source) {
        p.advance_by(len);
        return Ok(ChatMessageContentPart::Url { url });
    }

    if let Some((len, json)) = parse_json(source) {
        p.advance_by(len);
        return Ok(ChatMessageContentPart::Json { json });
    }

    let mut index = 0;

    while index < source.len() {
        if let Some(end) = code_end(source, index) {
            index = end;
            continue;
        }

        let previous = source[..index].chars().next_back();
        let boundary = previous.is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_');

        if index > 0
            && (source[index..].starts_with("<think>")
                || boundary && parse_url(&source[index..]).is_some()
                || parse_json(&source[index..]).is_some())
        {
            break;
        }

        index += source[index..].chars().next().unwrap().len_utf8();
    }

    if index == 0 {
        return Err(nim_error::Diagnostic::message("message content is empty"));
    }

    let text = source[..index].to_string();
    p.advance_by(index);
    Ok(ChatMessageContentPart::Text { text })
}

fn parse_url(source: &str) -> Option<(usize, url::Url)> {
    if !source.starts_with("http://") && !source.starts_with("https://") {
        return None;
    }

    let end = source.find(|c: char| c.is_whitespace() || c == '<').unwrap_or(source.len());
    let value = source[..end].trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '}', ']']);
    let url = url::Url::parse(value).ok()?;
    Some((value.len(), url))
}

fn parse_json(source: &str) -> Option<(usize, serde_json::Value)> {
    if !source.starts_with('{') && !source.starts_with('[') {
        return None;
    }

    let mut values = serde_json::Deserializer::from_str(source).into_iter::<serde_json::Value>();
    let value = values.next()?.ok()?;
    Some((values.byte_offset(), value))
}

fn code_end(source: &str, index: usize) -> Option<usize> {
    let rest = &source[index..];
    let marker = rest.chars().next()?;

    if marker != '`' && marker != '~' {
        return None;
    }

    let count = rest.chars().take_while(|c| *c == marker).count();
    let line_start = source[..index].rfind('\n').map_or(0, |i| i + 1);

    if count >= 3 && source[line_start..index].trim().is_empty() {
        let mut offset = source[index..].find('\n').map_or(source.len(), |i| index + i + 1);

        while offset < source.len() {
            let end = source[offset..].find('\n').map_or(source.len(), |i| offset + i);
            let line = source[offset..end].trim_end_matches('\r');
            let trimmed = line.trim_start_matches([' ', '\t']);
            let indent = line.len() - trimmed.len();
            let closing = trimmed.chars().take_while(|c| *c == marker).count();

            if indent <= 3 && closing >= count && trimmed[closing..].trim_matches([' ', '\t']).is_empty() {
                return Some((end + 1).min(source.len()));
            }

            offset = (end + 1).min(source.len());
        }

        return Some(source.len());
    }

    if marker == '`' {
        let delimiter = &rest[..count];
        return Some(
            rest[count..]
                .find(delimiter)
                .map_or(source.len(), |i| index + count + i + count),
        );
    }

    None
}
