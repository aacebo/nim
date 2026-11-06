mod all;
mod artifacts;
mod chats;
mod memories;
mod messages;

use nim_store::{PgPool, Storage};

#[derive(Copy, Clone)]
pub struct SearchService<'a> {
    storage: Storage<'a>,
}

impl<'a> SearchService<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            storage: Storage::new(pool),
        }
    }
}
