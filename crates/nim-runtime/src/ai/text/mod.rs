mod classify;

pub use classify::*;

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
