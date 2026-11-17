use askama::Template;

#[derive(Template)]
#[template(path = "chat/chat.html")]
pub(super) struct ChatView;
