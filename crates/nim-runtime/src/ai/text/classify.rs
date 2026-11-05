use nim_ai::{models::ModelRef, types::Annotation};

use crate::{Error, TaskOptions, TaskSource, ai::TextPipelines, task_run};

#[derive(Debug, Clone)]
pub struct TextClassifyOptions {
    pub model: ModelRef,
    pub min_score: f64,
    pub limit: Option<usize>,
    pub labels: Option<Vec<String>>,
    pub multi_label: bool,
    pub hypothesis: Option<String>,
    pub task: TaskOptions,
}

impl Default for TextClassifyOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::text::classify::Options::default();

        Self {
            model: options.model,
            min_score: options.min_score,
            limit: options.limit,
            labels: options.labels,
            multi_label: options.multi_label,
            hypothesis: options.hypothesis_template,
            task: Default::default(),
        }
    }
}

impl From<TextClassifyOptions> for nim_ai::pipelines::text::classify::Options {
    fn from(options: TextClassifyOptions) -> Self {
        Self {
            model: options.model,
            min_score: options.min_score,
            limit: options.limit,
            labels: options.labels,
            multi_label: options.multi_label,
            hypothesis_template: options.hypothesis,
        }
    }
}

impl<'a> TextPipelines<'a> {
    pub async fn classify<T>(&self, input: &[T], options: TextClassifyOptions) -> Result<Vec<Vec<Annotation>>, Error>
    where
        T: AsRef<str>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "text_classify", &options, {
            let input = input.iter().map(|v| v.as_ref().as_bytes()).collect::<Vec<_>>();
            nim_ai::pipelines::text::classify::run_with(&input, options.into())
        })?;

        Ok(out)
    }
}
