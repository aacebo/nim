use nim_ai::{
    SplitOptions,
    models::ModelRef,
    types::{TextOutput, Word},
};

use crate::{TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextSplitOptions {
    pub model: ModelRef,
    pub split: SplitOptions,
    pub task: TaskOptions,
}

impl Default for TextSplitOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::split::Options::default();

        Self {
            model: options.model,
            split: options.split,
            task: Default::default(),
        }
    }
}

impl From<TextSplitOptions> for nim_ai::pipelines::text::split::Options {
    fn from(options: TextSplitOptions) -> Self {
        Self {
            model: options.model,
            split: options.split,
        }
    }
}

impl<'a> TextPipelines<'a> {
    pub async fn split(&self, input: &str, options: TextSplitOptions) -> nim_error::Result<Vec<TextOutput>> {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_split", &options, {
            nim_ai::pipelines::text::split::run_with(input, options.into()).await
        })?;

        Ok(out)
    }

    pub async fn split_delimited(&self, input: &str, options: TextSplitOptions) -> nim_error::Result<Vec<Word>> {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_split_delimited", &options, {
            nim_ai::pipelines::text::split::run_delimited_with(input, options.into()).await
        })?;

        Ok(out)
    }
}
