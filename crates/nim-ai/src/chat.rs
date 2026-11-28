use nim_chat::Chat;

use crate::Context;

pub trait Complete: Send + Sync {
    fn complete<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        chat: Chat,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<nim_chat::ChatAgentMessage>>;
}
