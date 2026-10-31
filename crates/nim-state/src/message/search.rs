use nim_store::{MessageFilter, SearchItem};

use crate::{Error, Message, MessageService};

impl<'a> MessageService<'a> {
    #[tracing::instrument(skip(self))]
    pub async fn search(&self, embedding: &[f32], filter: MessageFilter) -> Result<Vec<SearchItem<Message>>, Error> {
        let rows = self.storage.messages().search(embedding, filter).await?;

        Ok(rows
            .into_iter()
            .map(|row| SearchItem::new(row.score, row.into_inner().into()))
            .collect())
    }
}
