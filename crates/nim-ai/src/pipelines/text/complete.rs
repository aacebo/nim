use crate::tasks;
use crate::{
    Chat, ChatAssistantMessage,
    models::{ModelRef, load},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    pub model: ModelRef,

    #[serde(flatten)]
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
pub fn run(options: Options) -> nim_error::Result<ChatAssistantMessage> {
    let model = load(&options.model)?;
    let capable = model.as_complete().ok_or_else(|| model.cannot("complete"))?;
    tasks::complete(capable, &model.context(), options.chat)
}
