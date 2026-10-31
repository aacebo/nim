use crate::{Error, Message, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_parent_id(&self, parent_id: MessageId) -> Result<Vec<Message>, Error> {
        let rows = self.storage.messages().find_by_parent_id(parent_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
