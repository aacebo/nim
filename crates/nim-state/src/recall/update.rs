use crate::{Error, Recall, RecallId, RecallService, RecallSource, RecallTrigger};

#[derive(Debug, Default)]
pub struct RecallUpdate {
    pub trigger: Option<RecallTrigger>,
    pub source: Option<RecallSource>,
    pub query: Option<String>,
}

impl<'a> RecallService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn update(&self, id: RecallId, value: RecallUpdate) -> Result<Recall, Error> {
        let mut row = self.storage.recalls().find_by_id(id.into()).await?;

        if let Some(value) = value.trigger {
            row.trigger = value.into();
        }

        if let Some(value) = value.source {
            row.source = value.into();
        }

        if let Some(value) = value.query {
            row.query = Some(value);
        }

        Ok(self.storage.recalls().update(row).await?.into())
    }
}
