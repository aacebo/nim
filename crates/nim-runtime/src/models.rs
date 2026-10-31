use nim_ai::models;
use nim_state as state;

pub fn persist(pool: nim_store::PgPool) -> Result<nim_ai::Subscription, crate::Error> {
    let runtime = tokio::runtime::Handle::try_current().map_err(|error| nim_ai::Error::custom("runtime", error))?;

    Ok(models::on_load(move |model, config| {
        let model = model.clone();
        let pool = pool.clone();
        let name = match &model {
            models::ModelRef::Local(models::LocalModel::Path(resource)) => resource.uri.to_string(),
            _ => model.name(),
        };
        let ty = config.model_type.to_string();
        let architectures = config.architectures.iter().map(ToString::to_string).collect::<Vec<_>>();

        runtime.spawn(async move {
            for attempt in 0..3 {
                let result = async {
                    let source = match &model {
                        models::ModelRef::Local(local @ models::LocalModel::Hub(id)) => {
                            let resource = local.repository()?.resolve(std::path::Path::new(""))?;

                            state::ModelSource::Local(state::LocalModelSource::Hub {
                                id: id.to_string(),
                                resource: Some(state::ModelResource {
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
    })?)
}
