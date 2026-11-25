use nim_ai::{models::ModelRef, types::Artifact};

use crate::{TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextSummarizeOptions {
    pub model: ModelRef,
    pub embedding: nim_ai::pipelines::text::embeddings::Options,
    pub beams: Option<usize>,
    pub max_tokens: Option<usize>,
    pub task: TaskOptions,
}

impl Default for TextSummarizeOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::summarize::Options::default();

        Self {
            model: options.model,
            embedding: options.embedding,
            beams: options.beams,
            max_tokens: options.max_tokens,
            task: Default::default(),
        }
    }
}

impl From<TextSummarizeOptions> for nim_ai::pipelines::text::summarize::Options {
    fn from(options: TextSummarizeOptions) -> Self {
        Self {
            model: options.model,
            embedding: options.embedding,
            beams: options.beams,
            max_tokens: options.max_tokens,
        }
    }
}

impl<'a> TextPipelines<'a> {
    pub async fn summarize<T>(&self, input: &[T], options: TextSummarizeOptions) -> nim_error::Result<Vec<Artifact>>
    where
        T: AsRef<str>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_summarize", &options, {
            let input = input.iter().map(|v| v.as_ref().as_bytes()).collect::<Vec<_>>();
            nim_ai::pipelines::text::summarize::run_with(&input, options.into()).await
        })?;

        Ok(out)
    }
}
