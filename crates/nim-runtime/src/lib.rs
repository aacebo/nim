pub mod ai;
mod models;
mod render;
mod tasks;
mod transform;

pub use render::*;
pub use tasks::*;
pub use transform::*;

pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct Runtime<'a> {
    _pipelines: ai::Pipelines<'a>,
}

impl<'a> Runtime<'a> {
    pub fn new(pool: &'a nim_store::PgPool, signals: std::sync::Arc<nim_signals::Client>) -> Self {
        Self {
            _pipelines: ai::Pipelines::new(pool, signals),
        }
    }

    pub fn pipelines(&self) -> &ai::Pipelines<'a> {
        &self._pipelines
    }
}
