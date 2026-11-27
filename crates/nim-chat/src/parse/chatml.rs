use crate::{ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, Parse, ParseMode, Parser, Tag, ToolCall};

pub fn parse_message(p: &mut Parser<'_>) -> nim_error::Result<ChatMessage> {
    Ok(parse_agent_message(p)?.into())
}

pub fn parse_agent_message(p: &mut Parser<'_>) -> nim_error::Result<ChatAgentMessage> {
    let mode = *p.mode();
    let src = p.take_remaining();
    let mut src = src.trim();
    let mut thinking = None;

    if src.starts_with(Tag::Think.open()) {
        let mut parser = Parser::from_src(src, mode);
        parser.parse_open_tag(Tag::Think)?;

        match parser.take_until_exact(Tag::Think.close()) {
            Some(value) => {
                parser.parse_close_tag(Tag::Think)?;
                thinking = Some(value.trim().to_string());
                src = parser.remaining();
            }
            None if mode == ParseMode::Strict => {
                return Err(nim_error::Diagnostic::message("unterminated Qwen reasoning block"));
            }
            None => src = parser.remaining(),
        }
    }

    let anchor = Parser::from_src(src, mode);
    let mut scanner = anchor.clone();
    let mut text = String::new();
    let mut tool_calls = Vec::new();

    while !scanner.is_eof() {
        text.push_str(scanner.take_while(|ch| !matches!(ch, '<' | '`' | '~')));

        if scanner.is_eof() {
            break;
        }

        let offset = scanner.span().start();

        if let Some(end) = anchor.seek_code_end(offset) {
            let code = scanner
                .take_to_offset(end)
                .ok_or_else(|| scanner.error("invalid code block"))?;
            text.push_str(code);
            continue;
        }

        if scanner.starts_with(Tag::Widget.open()) {
            let mut widget = scanner.clone();
            widget.parse_open_tag(Tag::Widget)?;
            widget.take_while(char::is_whitespace);

            if widget.parse_json().is_ok() {
                widget.take_while(char::is_whitespace);

                if widget.parse_close_tag(Tag::Widget).is_ok() {
                    let text_end = widget.span().start();
                    let widget_text = scanner
                        .take_to_offset(text_end)
                        .ok_or_else(|| scanner.error("invalid widget"))?;
                    text.push_str(widget_text);
                    scanner = widget;
                    continue;
                }
            }
        }

        if !scanner.starts_with(Tag::ToolCall.open()) {
            text.push(scanner.take_char().expect("scanner is not at eof"));
            continue;
        }

        let mut tool_call = scanner.clone();
        tool_call.parse_open_tag(Tag::ToolCall)?;

        let Some(raw) = tool_call.take_until_exact(Tag::ToolCall.close()) else {
            if mode == ParseMode::Strict {
                return Err(nim_error::Diagnostic::message("unterminated Qwen tool call"));
            }

            text.push_str(scanner.remaining());
            break;
        };

        tool_call.parse_close_tag(Tag::ToolCall)?;

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
                text.push_str(Tag::ToolCall.open());
                text.push_str(raw);
                text.push_str(Tag::ToolCall.close());
            }
        }

        scanner = tool_call;
    }

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
    let src = p.take_remaining();
    let value: serde_json::Value = serde_json::from_str(src.trim())?;
    let object = value
        .as_object()
        .ok_or_else(|| nim_error::Diagnostic::message("tool call must be an object"))?;
    let name = object
        .get("name")
        .or_else(|| object.get("function").and_then(|v| v.get("name")))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| nim_error::Diagnostic::message("tool call is missing a name"))?
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
                "tool arguments must be an object",
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
