use nim_ai::{ChatAssistantMessage, ChatOptions, models::ModelRef};

use crate::{TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextChatOptions {
    pub model: ModelRef,
    pub chat: ChatOptions,
    pub task: TaskOptions,
}

impl Default for TextChatOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::chat::Options::default();

        Self {
            model: options.model,
            chat: options.chat,
            task: Default::default(),
        }
    }
}

impl From<TextChatOptions> for nim_ai::pipelines::text::chat::Options {
    fn from(options: TextChatOptions) -> Self {
        Self {
            model: options.model,
            chat: options.chat,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextChatSummarizeOptions {
    pub model: ModelRef,
    pub chat: ChatOptions,
    pub task: TaskOptions,
}

impl Default for TextChatSummarizeOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::chat::Options::default();

        Self {
            model: options.model,
            chat: options.chat,
            task: Default::default(),
        }
    }
}

impl From<TextChatSummarizeOptions> for nim_ai::pipelines::text::chat::Options {
    fn from(options: TextChatSummarizeOptions) -> Self {
        Self {
            model: options.model,
            chat: options.chat,
        }
    }
}

impl<'a> TextPipelines<'a> {
    pub async fn chat(&self, options: TextChatOptions) -> nim_error::Result<ChatAssistantMessage> {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_chat", &options, {
            nim_ai::pipelines::text::chat::run(options.into())
        })?;

        Ok(out)
    }
}
