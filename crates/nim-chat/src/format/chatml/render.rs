use crate::{
    Chat, ChatAgentMessage, ChatMessage, ChatMessageRole, ChatSystemMessage, ChatToolMessage, ChatUserMessage, Document, Tag,
    Tool, ToolCall,
    render::{Formatter, Render},
};

const IM_START: &str = "<|im_start|>";
const IM_END: &str = "<|im_end|>";

pub fn render_chat(f: &mut Formatter, chat: &Chat) -> nim_error::Result<()> {
    if !chat.tools.is_empty() {
        render_header(f, "system", None)?;
        f.writeln("<tools>").map_err(nim_error::IntoDiagnostic::into_diagnostic)?;

        for tool in &chat.tools {
            render_tool(f, tool)?;
            f.writeln("").map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
        }

        f.writeln(format!("</tools>{IM_END}"))
            .map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    }

    if !chat.documents.is_empty() {
        render_header(f, "system", None)?;
        f.writeln("<documents>").map_err(nim_error::IntoDiagnostic::into_diagnostic)?;

        for document in &chat.documents {
            render_document(f, document)?;
            f.writeln("").map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
        }

        f.writeln(format!("</documents>{IM_END}"))
            .map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    }

    for message in &chat.messages {
        render_message(f, message)?;
    }

    if chat
        .messages
        .last()
        .is_none_or(|message| !matches!(message, ChatMessage::Agent(_)))
    {
        render_header(f, "assistant", None)?;
    }

    Ok(())
}

pub fn render_message(f: &mut Formatter, message: &ChatMessage) -> nim_error::Result<()> {
    match message {
        ChatMessage::System(message) => render_system_message(f, message),
        ChatMessage::User(message) => render_user_message(f, message),
        ChatMessage::Agent(message) => render_agent_message(f, message),
        ChatMessage::Tool(message) => render_tool_message(f, message),
    }
}

fn render_system_message(f: &mut Formatter, message: &ChatSystemMessage) -> nim_error::Result<()> {
    let role = match message.role {
        ChatMessageRole::System => "system",
        ChatMessageRole::Developer => "developer",
        _ => return Err(nim_error::Diagnostic::message("invalid system message role")),
    };

    render_header(f, role, message.name.as_deref())?;
    message.content.render(f)?;
    f.writeln(IM_END).map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

fn render_user_message(f: &mut Formatter, message: &ChatUserMessage) -> nim_error::Result<()> {
    render_header(f, "user", message.name.as_deref())?;
    message.content.render(f)?;
    f.writeln(IM_END).map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

pub fn render_agent_message(f: &mut Formatter, message: &ChatAgentMessage) -> nim_error::Result<()> {
    render_header(f, "assistant", message.name.as_deref())?;

    if let Some(content) = &message.content {
        content.render(f)?;
    }

    for call in &message.tool_calls {
        render_tool_call(f, call)?;
    }

    f.writeln(IM_END).map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

fn render_tool_message(f: &mut Formatter, message: &ChatToolMessage) -> nim_error::Result<()> {
    render_header(f, "tool", None)?;
    f.write(Tag::ToolResponse.open())
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    f.write(serde_json::to_string(message)?)
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    f.writeln(format!("{}{}", Tag::ToolResponse.close(), IM_END))
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

pub fn render_tool(f: &mut Formatter, tool: &Tool) -> nim_error::Result<()> {
    f.write(serde_json::to_string(tool)?)
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

pub fn render_tool_call(f: &mut Formatter, call: &ToolCall) -> nim_error::Result<()> {
    f.write(Tag::ToolCall.open())
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    f.write(serde_json::to_string(call)?)
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    f.write(Tag::ToolCall.close())
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

pub fn render_document(f: &mut Formatter, document: &Document) -> nim_error::Result<()> {
    f.write(serde_json::to_string(document)?)
        .map_err(nim_error::IntoDiagnostic::into_diagnostic)
}

fn render_header(f: &mut Formatter, role: &str, name: Option<&str>) -> nim_error::Result<()> {
    f.write(IM_START).map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    f.write(role).map_err(nim_error::IntoDiagnostic::into_diagnostic)?;

    if let Some(name) = name {
        f.write(format!(" name={}", serde_json::to_string(name)?))
            .map_err(nim_error::IntoDiagnostic::into_diagnostic)?;
    }

    f.writeln("").map_err(nim_error::IntoDiagnostic::into_diagnostic)
}
