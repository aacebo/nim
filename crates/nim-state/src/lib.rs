mod error;
mod memory;
mod types;

pub use error::*;
pub use memory::*;
pub use types::*;

pub struct Services<'a> {
    pub memories: MemoryService<'a>,
}

impl<'a> Services<'a> {
    pub fn new(pool: &'a nim_store::PgPool) -> Self {
        Self {
            memories: MemoryService::new(pool),
        }
    }
}
