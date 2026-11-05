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
        let model = nim_ai::models::load(&options.model)?;
        let config = model.config().cloned();
        let source = match &options.model {
            nim_ai::models::ModelRef::Local(local @ nim_ai::models::LocalModel::Hub(id)) => {
                let resource = local.repository()?.resolve(std::path::Path::new(""))?;

                nim_state::ModelSource::Local(nim_state::LocalModelSource::Hub {
                    id: id.to_string(),
                    resource: Some(nim_state::ModelResource {
                        uri: resource.uri.to_string(),
                        format: resource.format.to_string(),
                        path: resource.path,
                        size: resource.size,
                    }),
                })
            }
            nim_ai::models::ModelRef::Local(nim_ai::models::LocalModel::Path(resource)) => {
                nim_state::ModelSource::Local(nim_state::LocalModelSource::Path {
                    uri: resource.uri.to_string(),
                    format: resource.format.to_string(),
                    path: resource.path.clone(),
                    size: resource.size,
                })
            }
            nim_ai::models::ModelRef::Remote(remote) => nim_state::ModelSource::Remote {
                provider: remote.provider.to_string(),
                id: remote.id.to_string(),
                base_url: remote.base_url.clone(),
            },
        };

        let model = self
            ._services
            .models()
            .upsert(nim_state::ModelCreate {
                ty: config.as_ref().map(|c| c.model_type.to_string()).unwrap_or("??".into()),
                name: options.model.to_string(),
                source,
                tasks: Default::default(),
                architectures: config
                    .map(|v| v.architectures.iter().map(ToString::to_string).collect::<Vec<_>>())
                    .unwrap_or_default(),
            })
            .await?;

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
