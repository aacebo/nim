use crate::{Message, MessageId, MessageService, PageResponse};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_history(&self, id: MessageId, count: usize) -> nim_error::Result<PageResponse<Message>> {
        let rows = self.storage.messages().find_history(id.into(), count).await?;
        rows.try_into()
    }
}
