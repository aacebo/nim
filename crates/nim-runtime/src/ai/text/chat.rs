use nim_ai::{models::Generation, models::ModelRef};

use crate::{TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextChatOptions {
    pub model: ModelRef,
    pub system: String,
    pub beams: Option<usize>,
    pub max_len: Option<usize>,
    pub task: TaskOptions,
}

impl Default for TextChatOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::chat::Options::default();

        Self {
            model: options.model,
            system: options.system,
            beams: options.beams,
            max_len: options.max_len,
            task: Default::default(),
        }
    }
}

impl From<TextChatOptions> for nim_ai::pipelines::text::chat::Options {
    fn from(options: TextChatOptions) -> Self {
        Self {
            model: options.model,
            system: options.system,
            beams: options.beams,
            max_len: options.max_len,
        }
    }
}

impl<'a> TextPipelines<'a> {
    pub async fn chat<T>(&self, input: &[T], options: TextChatOptions) -> nim_error::Result<Vec<Generation>>
    where
        T: AsRef<str>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_chat", &options, {
            let input = input.iter().map(|v| v.as_ref().as_bytes()).collect::<Vec<_>>();
            nim_ai::pipelines::text::chat::run_with(&input, options.into())
        })?;

        Ok(out)
    }
}
