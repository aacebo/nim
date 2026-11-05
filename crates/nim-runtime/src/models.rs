use crate::Error;

pub(crate) async fn assert(
    services: &nim_state::Services<'_>,
    model_ref: &nim_ai::models::ModelRef,
) -> Result<nim_state::Model, Error> {
    let model = nim_ai::models::load(model_ref)?;
    let config = model.config().cloned();
    let source = match model_ref {
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

    let model = services
        .models()
        .upsert(nim_state::ModelCreate {
            ty: config.as_ref().map(|c| c.model_type.to_string()).unwrap_or("??".into()),
            name: model_ref.to_string(),
            source,
            tasks: Default::default(),
            architectures: config
                .map(|v| v.architectures.iter().map(ToString::to_string).collect::<Vec<_>>())
                .unwrap_or_default(),
        })
        .await?;

    Ok(model)
}
