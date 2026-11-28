use crate::{
    Chat, ChatAgentMessage, ChatMessage, ChatMessageContent, ChatMessageContentPart, ChatMessageRole, ChatSystemMessage,
    ChatToolMessage, ChatUserMessage, Document, Parse, Parser, Tag, Tool, ToolCall,
};

const IM_START: &str = "<|im_start|>";
const IM_END: &str = "<|im_end|>";

pub fn parse_chat(p: &mut Parser<'_>) -> nim_error::Result<Chat> {
    let mut chat = Chat::default();

    while !p.is_eof() {
        p.take_while(char::is_whitespace);

        if p.is_eof() {
            break;
        }

        p.take_exact(IM_START)
            .ok_or_else(|| p.error("expected ChatML message start"))?;
        let header = p.take_until('\n').ok_or_else(|| p.error("expected ChatML message role"))?;
        p.take_char();

        if p.is_eof() && header == "assistant" {
            break;
        }

        let content = p
            .take_until_exact(IM_END)
            .ok_or_else(|| p.error("unterminated ChatML message"))?;
        p.take_exact(IM_END).expect("ChatML message end was found");
        let (role, name) = parse_header(header)?;

        match role {
            "system" if name.is_none() && content.trim_start().starts_with("<tools>") => chat.tools.extend(parse_tools(content)?),
            "system" if name.is_none() && content.trim_start().starts_with("<documents>") => {
                chat.documents.extend(parse_documents(content)?)
            }
            _ => chat.messages.push(parse_message_with_role(role, name, content)?),
        }
    }

    Ok(chat)
}

pub fn parse_message(p: &mut Parser<'_>) -> nim_error::Result<ChatMessage> {
    Ok(parse_agent_message(p)?.into())
}

fn parse_header(header: &str) -> nim_error::Result<(&str, Option<String>)> {
    let (role, attributes) = header.split_once(' ').unwrap_or((header, ""));

    if attributes.is_empty() {
        return Ok((role, None));
    }

    let name = attributes
        .strip_prefix("name=")
        .and_then(|value| serde_json::from_str(value).ok())
        .ok_or_else(|| nim_error::Diagnostic::message("invalid ChatML message name"))?;
    Ok((role, Some(name)))
}

fn parse_message_with_role(role: &str, name: Option<String>, src: &str) -> nim_error::Result<ChatMessage> {
    let mut p = Parser::from_src(src);

    match role {
        "system" | "developer" => Ok(ChatSystemMessage {
            role: if role == "developer" {
                ChatMessageRole::Developer
            } else {
                ChatMessageRole::System
            },
            name,
            content: ChatMessageContent::parse(&mut p)?,
            extra: Default::default(),
        }
        .into()),
        "user" => Ok(ChatUserMessage {
            name,
            content: ChatMessageContent::parse(&mut p)?,
            extra: Default::default(),
        }
        .into()),
        "assistant" | "agent" => Ok(parse_agent_message_with_name(&mut p, name)?.into()),
        "tool" if name.is_none() => parse_tool_message(src).map(Into::into),
        _ => Err(nim_error::Diagnostic::message("unsupported ChatML message role")),
    }
}

pub fn parse_agent_message(p: &mut Parser<'_>) -> nim_error::Result<ChatAgentMessage> {
    parse_agent_message_with_name(p, None)
}

fn parse_agent_message_with_name(p: &mut Parser<'_>, name: Option<String>) -> nim_error::Result<ChatAgentMessage> {
    let src = p.take_remaining();
    let mut scanner = Parser::from_src(src);
    let mut text = String::new();
    let mut tool_calls = Vec::new();

    while !scanner.is_eof() {
        let offset = scanner.span().start();

        if let Some(end) = scanner.seek_code_end(offset) {
            text.push_str(
                scanner
                    .take_to_offset(end)
                    .ok_or_else(|| scanner.error("invalid code block"))?,
            );
            continue;
        }

        if scanner.starts_with_tag() {
            let mut tool_call = scanner.clone();

            if tool_call.parse_open_tag(Tag::ToolCall).is_ok() {
                let Some(raw) = tool_call.take_until_exact(Tag::ToolCall.close()) else {
                    text.push_str(scanner.remaining());
                    break;
                };

                if tool_call.parse_close_tag(Tag::ToolCall).is_ok()
                    && let Ok(call) = serde_json::from_str::<ToolCall>(raw.trim())
                {
                    tool_calls.push(call);
                    scanner = tool_call;
                    continue;
                }
            }

            let mut content = scanner.clone();

            if ChatMessageContentPart::parse(&mut content).is_ok() {
                text.push_str(
                    scanner
                        .take_to_offset(content.span().start())
                        .ok_or_else(|| scanner.error("invalid message content"))?,
                );
                continue;
            }
        }

        text.push(scanner.take_char().expect("scanner is not at eof"));
    }

    let content = (!text.is_empty())
        .then(|| ChatMessageContent::parse(&mut Parser::from_src(&text)))
        .transpose()?;

    Ok(ChatAgentMessage {
        name,
        content,
        tool_calls,
        annotations: Vec::new(),
        extra: Default::default(),
    })
}

pub fn parse_tools(src: &str) -> nim_error::Result<Vec<Tool>> {
    parse_section(src, "tools")
}

pub fn parse_documents(src: &str) -> nim_error::Result<Vec<Document>> {
    parse_section(src, "documents")
}

fn parse_section<T: serde::de::DeserializeOwned>(src: &str, name: &str) -> nim_error::Result<Vec<T>> {
    let src = src.trim();
    let opening = format!("<{name}>");
    let closing = format!("</{name}>");
    let body = src
        .strip_prefix(&opening)
        .and_then(|value| value.strip_suffix(&closing))
        .ok_or_else(|| nim_error::Diagnostic::message(format!("invalid ChatML {name} section")))?;
    let mut p = Parser::from_src(body.trim());
    let mut values = Vec::new();

    while !p.is_eof() {
        p.take_while(char::is_whitespace);

        if p.is_eof() {
            break;
        }

        values.push(serde_json::from_value(p.parse_json()?)?);
    }

    Ok(values)
}

fn parse_tool_message(src: &str) -> nim_error::Result<ChatToolMessage> {
    let mut p = Parser::from_src(src.trim());
    p.parse_open_tag(Tag::ToolResponse)?;
    let raw = p
        .take_until_exact(Tag::ToolResponse.close())
        .ok_or_else(|| p.error("unterminated ChatML tool response"))?;
    p.parse_close_tag(Tag::ToolResponse)?;

    if !p.remaining().trim().is_empty() {
        return Err(p.error("invalid ChatML tool response"));
    }

    Ok(serde_json::from_str(raw.trim())?)
}
