use crate::{Model, ModelService};

impl<'a> ModelService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_name(&self, name: &str) -> nim_error::Result<Option<Model>> {
        self.storage
            .models()
            .find_by_name(name)
            .await?
            .map(TryInto::try_into)
            .transpose()
    }
}
