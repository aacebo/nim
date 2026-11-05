use nim_ai::{models::ModelRef, types::Artifact};

use crate::{Error, TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextEmbedOptions {
    pub model: ModelRef,
    pub task: TaskOptions,
}

impl Default for TextEmbedOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::embeddings::Options::default();

        Self {
            model: options.model,
            task: Default::default(),
        }
    }
}

impl From<TextEmbedOptions> for nim_ai::pipelines::text::embeddings::Options {
    fn from(options: TextEmbedOptions) -> Self {
        Self { model: options.model }
    }
}

impl<'a> TextPipelines<'a> {
    pub async fn embed<T>(&self, input: &[T], options: TextEmbedOptions) -> Result<Vec<Artifact>, Error>
    where
        T: AsRef<str>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_embeddings", &options, {
            let input = input.iter().map(|v| v.as_ref().as_bytes()).collect::<Vec<_>>();
            nim_ai::pipelines::text::embeddings::run_with(&input, options.into())
        })?;

        Ok(out)
    }
}
