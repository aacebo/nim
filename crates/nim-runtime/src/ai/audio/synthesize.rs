use nim_ai::{SynthesizeOptions, models::ModelRef};

use crate::{TaskOptions, TaskSource, ai::AudioPipelines, task_run};

#[derive(Debug, Clone)]
pub struct AudioSynthesizeOptions {
    pub model: ModelRef,
    pub synthesize: SynthesizeOptions,
    pub task: TaskOptions,
}

impl Default for AudioSynthesizeOptions {
    fn default() -> Self {
        let options = nim_ai::pipelines::audio::synthesize::Options::default();

        Self {
            model: options.model,
            synthesize: options.synthesize,
            task: Default::default(),
        }
    }
}

impl From<AudioSynthesizeOptions> for nim_ai::pipelines::audio::synthesize::Options {
    fn from(options: AudioSynthesizeOptions) -> Self {
        Self {
            model: options.model,
            synthesize: options.synthesize,
        }
    }
}

impl<'a> AudioPipelines<'a> {
    pub async fn synthesize<T>(&self, input: &[T], options: AudioSynthesizeOptions) -> nim_error::Result<Vec<u8>>
    where
        T: AsRef<str>,
    {
        let model = crate::models::assert(&self._services, &options.model).await?;
        let mut options = options;

        if !options.task.sources.contains(&TaskSource::Model(model.id)) {
            options.task.sources.push(TaskSource::Model(model.id));
        }

        let out = task_run!(self, "audio_synthesize", &options, {
            let input = input.iter().map(|value| value.as_ref()).collect::<Vec<_>>();
            nim_ai::pipelines::audio::synthesize::run_with(&input, options.into()).await
        })?;

        Ok(out)
    }
}
