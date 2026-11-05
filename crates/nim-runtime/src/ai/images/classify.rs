use nim_ai::{models::ModelRef, types::Annotation};

use crate::{Error, TaskOptions, TaskSource, ai::ImagePipelines, task_run};

#[derive(Debug, Clone)]
pub struct ImageClassifyOptions {
    pub model: ModelRef,
    pub min_score: f64,
    pub limit: Option<usize>,
    pub labels: Option<Vec<String>>,
    pub multi_label: bool,
    pub hypothesis: Option<String>,
    pub task: TaskOptions,
}

impl Default for ImageClassifyOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::images::classify::Options::default();

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

impl From<ImageClassifyOptions> for nim_ai::pipelines::images::classify::Options {
    fn from(options: ImageClassifyOptions) -> Self {
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

impl<'a> ImagePipelines<'a> {
    pub async fn classify<T>(&self, input: &[T], options: ImageClassifyOptions) -> Result<Vec<Vec<Annotation>>, Error>
    where
        T: AsRef<[u8]>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "images_classify", &options, {
            let input = input.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
            nim_ai::pipelines::images::classify::run_with(&input, options.into())
        })?;

        Ok(out)
    }
}
