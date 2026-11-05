use nim_ai::{models::ModelRef, types::Annotation};

use crate::{Error, TaskOptions, TaskSource, ai::AudioPipelines, task_run};

#[derive(Debug, Clone)]
pub struct AudioClassifyOptions {
    pub model: ModelRef,
    pub min_score: f64,
    pub limit: Option<usize>,
    pub labels: Option<Vec<String>>,
    pub multi_label: bool,
    pub hypothesis: Option<String>,
    pub task: TaskOptions,
}

impl Default for AudioClassifyOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::audio::classify::Options::default();

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

impl From<AudioClassifyOptions> for nim_ai::pipelines::audio::classify::Options {
    fn from(options: AudioClassifyOptions) -> Self {
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

impl<'a> AudioPipelines<'a> {
    pub async fn classify<T>(&self, input: &[T], options: AudioClassifyOptions) -> Result<Vec<Vec<Annotation>>, Error>
    where
        T: AsRef<[u8]>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "audio_classify", &options, {
            let input = input.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
            nim_ai::pipelines::audio::classify::run_with(&input, options.into())
        })?;

        Ok(out)
    }
}
