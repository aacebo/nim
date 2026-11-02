use crate::{ChatId, Error, Message, MessageService, Page};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: ChatId, page: Page) -> Result<Vec<Message>, Error> {
        let rows = self.storage.messages().find_by_chat_id(chat_id.into(), page).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
