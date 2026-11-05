use nim_ai::{models::ModelRef, types::Artifact};

use crate::{Error, TaskOptions, TaskSource, ai::AudioPipelines, task_run};

#[derive(Debug, Clone)]
pub struct AudioCaptionsOptions {
    pub model: ModelRef,
    pub beams: Option<usize>,
    pub max_len: Option<usize>,
    pub task: TaskOptions,
}

impl Default for AudioCaptionsOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::audio::captions::Options::default();

        Self {
            model: options.model,
            beams: options.beams,
            max_len: options.max_len,
            task: Default::default(),
        }
    }
}

impl From<AudioCaptionsOptions> for nim_ai::pipelines::audio::captions::Options {
    fn from(options: AudioCaptionsOptions) -> Self {
        Self {
            model: options.model,
            beams: options.beams,
            max_len: options.max_len,
        }
    }
}

impl<'a> AudioPipelines<'a> {
    pub async fn captions<T>(&self, input: &[T], options: AudioCaptionsOptions) -> Result<Vec<Artifact>, Error>
    where
        T: AsRef<[u8]>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "audio_captions", &options, {
            let input = input.iter().map(|v| v.as_ref()).collect::<Vec<_>>();
            nim_ai::pipelines::audio::captions::run_with(&input, options.into())
        })?;

        Ok(out)
    }
}
