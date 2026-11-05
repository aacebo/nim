pub mod ai;
mod error;
mod tasks;

pub use error::*;
pub use tasks::*;

pub struct Runtime<'a> {
    _pipelines: ai::Pipelines<'a>,
}

impl<'a> Runtime<'a> {
    pub fn new(pool: &'a nim_store::PgPool, signals: std::sync::Arc<nim_signals::Client>) -> Self {
        Self {
            _pipelines: ai::Pipelines::new(pool, signals),
        }
    }
}
