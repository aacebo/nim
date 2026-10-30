use crate::{Error, Model, ModelId, ModelService, ModelSource, ModelWeights};

#[derive(Debug, Default)]
pub struct ModelUpdate {
    pub ty: Option<String>,
    pub name: Option<String>,
    pub tasks: Option<Vec<String>>,
    pub architectures: Option<Vec<String>>,
    pub source: Option<ModelSource>,
    pub weights: Option<Vec<ModelWeights>>,
}

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: ModelId, value: ModelUpdate) -> Result<Model, Error> {
        let mut row = self.storage.models().find_by_id(id.into()).await?;

        if let Some(value) = value.ty {
            row.ty = value;
        }

        if let Some(value) = value.name {
            row.name = value;
        }

        if let Some(value) = value.tasks {
            row.tasks = value;
        }

        if let Some(value) = value.architectures {
            row.architectures = value;
        }

        if let Some(value) = value.source {
            row.source = nim_store::Json(value);
        }

        if let Some(value) = value.weights {
            row.weights = nim_store::Json(value);
        }

        Ok(self.storage.models().update(row).await?.into())
    }
}
