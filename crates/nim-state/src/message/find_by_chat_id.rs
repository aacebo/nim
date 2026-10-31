use crate::{ChatId, Error, Message, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_by_chat_id(&self, chat_id: ChatId) -> Result<Vec<Message>, Error> {
        let rows = self.storage.messages().find_by_chat_id(chat_id.into()).await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
