pub mod ai;
mod models;
mod render;
mod tasks;
mod transform;

pub use tasks::*;

pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Render {
    type Context<'a>
    where
        Self: 'a;

    fn render<'a>(&'a self, ctx: &'a Self::Context<'a>) -> BoxFuture<'a, nim_error::Result<String>>;
}

pub trait Transform {
    type Output;
    type Context<'a>
    where
        Self: 'a;

    fn transform<'a>(&'a self, ctx: &'a Self::Context<'a>) -> BoxFuture<'a, nim_error::Result<Self::Output>>;
}

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
