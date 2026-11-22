use nim_ai::{Chat, ChatAssistantMessage, models::ModelRef};

use crate::{TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextCompleteOptions {
    pub model: ModelRef,
    pub chat: Chat,
    pub task: TaskOptions,
}

impl Default for TextCompleteOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::complete::Options::default();

        Self {
            model: options.model,
            chat: options.chat,
            task: Default::default(),
        }
    }
}

impl From<TextCompleteOptions> for nim_ai::pipelines::text::complete::Options {
    fn from(options: TextCompleteOptions) -> Self {
        Self {
            model: options.model,
            chat: options.chat,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextCompleteSummarizeOptions {
    pub model: ModelRef,
    pub chat: Chat,
    pub task: TaskOptions,
}

impl Default for TextCompleteSummarizeOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::complete::Options::default();

        Self {
            model: options.model,
            chat: options.chat,
            task: Default::default(),
        }
    }
}

impl From<TextCompleteSummarizeOptions> for nim_ai::pipelines::text::complete::Options {
    fn from(options: TextCompleteSummarizeOptions) -> Self {
        Self {
            model: options.model,
            chat: options.chat,
        }
    }
}

impl<'a> TextPipelines<'a> {
    pub async fn complete(&self, options: TextCompleteOptions) -> nim_error::Result<ChatAssistantMessage> {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_complete", &options, {
            nim_ai::pipelines::text::complete::run(options.into())
        })?;

        Ok(out)
    }
}
