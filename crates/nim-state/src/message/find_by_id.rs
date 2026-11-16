use crate::{Message, MessageId, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_id(&self, id: MessageId) -> nim_error::Result<Message> {
        self.storage.messages().find_by_id(id.into()).await?.try_into()
    }
}
