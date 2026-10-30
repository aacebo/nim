pub use nim_ai::*;

use crate::{state, store};

pub fn register_model_observer(pool: store::PgPool) -> Result<(), crate::Error> {
    Ok(models::observe(ModelObserver::new(pool))?)
}

pub struct ModelObserver {
    pool: store::PgPool,
    runtime: tokio::runtime::Handle,
}

impl ModelObserver {
    pub fn new(pool: store::PgPool) -> Self {
        Self {
            pool,
            runtime: tokio::runtime::Handle::current(),
        }
    }
}

impl models::Observer for ModelObserver {
    fn on_load_success(&self, model: &models::ModelRef, config: &models::ModelConfig) {
        let model = model.clone();
        let pool = self.pool.clone();
        let name = model.name();
        let ty = config.model_type.to_string();
        let architectures = config.architectures.iter().map(ToString::to_string).collect::<Vec<_>>();

        self.runtime.spawn(async move {
            for attempt in 0..3 {
                let result = async {
                    let source = match &model {
                        models::ModelRef::Local(models::LocalModel::Hub(id)) => {
                            state::ModelSource::Local(state::LocalModelSource::Hub {
                                id: id.to_string(),
                                resource: id.cached_resource()?.map(|resource| state::ModelResource {
                                    uri: resource.uri.to_string(),
                                    format: resource.format.to_string(),
                                    path: resource.path,
                                    size: resource.size,
                                }),
                            })
                        }
                        models::ModelRef::Local(models::LocalModel::Path(resource)) => {
                            state::ModelSource::Local(state::LocalModelSource::Path {
                                uri: resource.uri.to_string(),
                                format: resource.format.to_string(),
                                path: resource.path.clone(),
                                size: resource.size,
                            })
                        }
                        models::ModelRef::Remote(remote) => state::ModelSource::Remote {
                            provider: remote.provider.to_string(),
                            id: remote.id.to_string(),
                            base_url: remote.base_url.clone(),
                        },
                    };

                    let model = state::Services::new(&pool)
                        .models()
                        .upsert(state::ModelCreate {
                            ty: ty.clone(),
                            name: name.clone(),
                            tasks: Vec::new(),
                            architectures: architectures.clone(),
                            source,
                            weights: Vec::new(),
                        })
                        .await?;

                    Ok::<_, crate::Error>(model)
                }
                .await;

                match result {
                    Ok(_) => return,
                    Err(error) => {
                        tracing::error!(target: "nim_state", %name, %error, "failed to register loaded model");
                        if attempt < 2 {
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        });
    }
}
