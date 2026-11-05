mod classify;
mod embeddings;
mod summarize;

pub use classify::*;
pub use embeddings::*;
pub use summarize::*;

pub struct TextPipelines<'a> {
    _signals: std::sync::Arc<nim_signals::Client>,
    _services: nim_state::Services<'a>,
}

impl<'a> TextPipelines<'a> {
    pub fn new(pool: &'a nim_store::PgPool, signals: std::sync::Arc<nim_signals::Client>) -> Self {
        Self {
            _signals: signals,
            _services: nim_state::Services::new(pool),
        }
    }
}
