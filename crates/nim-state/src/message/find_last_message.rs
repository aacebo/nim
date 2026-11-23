use crate::{ChatId, Message, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn find_last_message(&self, chat_id: ChatId) -> nim_error::Result<Option<Message>> {
        self.storage
            .messages()
            .find_last_message(chat_id.into())
            .await?
            .map(TryInto::try_into)
            .transpose()
    }
}
