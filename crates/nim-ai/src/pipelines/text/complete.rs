use nim_chat::ChatAgentMessage;

use crate::tasks;
use crate::{
    Chat,
    models::{ModelRef, load},
};

#[derive(Debug, Clone)]
pub struct Options {
    pub model: ModelRef,
    pub chat: Chat,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            model: super::defaults::complete(),
            chat: Default::default(),
        }
    }
}

#[tracing::instrument(skip_all, err)]
pub async fn run(options: Options) -> nim_error::Result<ChatAgentMessage> {
    let model = load(&options.model).await?;
    let capable = model.as_complete().ok_or_else(|| model.cannot("complete"))?;
    tasks::complete(capable, &model.context(), options.chat).await
}
