use nim_ai::{models::ModelRef, types::Artifact};

use crate::{Error, TaskOptions, TaskSource, ai::ImagePipelines, task_run};

#[derive(Debug, Clone)]
pub struct ImageCaptionsOptions {
    pub model: ModelRef,
    pub beams: Option<usize>,
    pub max_len: Option<usize>,
    pub task: TaskOptions,
}

impl Default for ImageCaptionsOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::images::captions::Options::default();

        Self {
            model: options.model,
            beams: options.beams,
            max_len: options.max_len,
            task: Default::default(),
        }
    }
}

impl From<ImageCaptionsOptions> for nim_ai::pipelines::images::captions::Options {
    fn from(options: ImageCaptionsOptions) -> Self {
        Self {
            model: options.model,
            beams: options.beams,
            max_len: options.max_len,
        }
    }
}

impl<'a> ImagePipelines<'a> {
    pub async fn captions<T>(&self, input: &[T], options: ImageCaptionsOptions) -> Result<Vec<Artifact>, Error>
    where
        T: AsRef<[u8]>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "images_captions", &options, {
            let input = input.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
            nim_ai::pipelines::images::captions::run_with(&input, options.into())
        })?;

        Ok(out)
    }
}
