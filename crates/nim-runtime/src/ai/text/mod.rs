mod classify;

pub use classify::*;

use nim_state::Services;

pub struct TextPipelines<'a> {
    _services: Services<'a>,
    _signals: std::sync::Arc<nim_signals::Client>,
}

impl<'a> TextPipelines<'a> {
    pub fn new(pool: &'a nim_store::PgPool, signals: std::sync::Arc<nim_signals::Client>) -> Self {
        Self {
            _services: Services::new(pool),
            _signals: signals,
        }
    }
}
