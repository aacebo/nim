use nim_ai::{Chat, ChatAssistantMessage, models::ModelRef};

use crate::{TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextCompleteOptions {
    pub model: ModelRef,
    pub chat: Chat,
    pub task: TaskOptions,
    pub instructions: Option<Vec<String>>,
}

impl Default for TextCompleteOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::complete::Options::default();

        Self {
            model: options.model,
            chat: options.chat,
            task: Default::default(),
            instructions: Default::default(),
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

impl<'a> TextPipelines<'a> {
    pub async fn complete(&self, options: TextCompleteOptions) -> nim_error::Result<ChatAssistantMessage> {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        if let Some(instructions) = &options.instructions {
            let mut messages = vec![
                nim_ai::chat::ChatSystemMessage {
                    content: nim_ai::chat::ChatMessageContent::Text(instructions.join("\n")),
                }
                .into(),
            ];

            messages.extend(options.chat.messages);
            options.chat.messages = messages;
        }

        let out = task_run!(self, "text_complete", &options, {
            nim_ai::pipelines::text::complete::run(options.into())
        })?;

        Ok(out)
    }
}
