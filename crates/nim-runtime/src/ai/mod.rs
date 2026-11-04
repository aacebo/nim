mod text;

pub use text::*;

pub struct Pipelines<'a> {
    _text: TextPipelines<'a>,
    _signals: std::sync::Arc<nim_signals::Client>,
}

impl<'a> Pipelines<'a> {
    pub fn new(pool: &'a nim_store::PgPool, signals: std::sync::Arc<nim_signals::Client>) -> Self {
        Self {
            _text: TextPipelines::new(pool, signals.clone()),
            _signals: signals,
        }
    }
}
