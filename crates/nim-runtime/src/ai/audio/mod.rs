mod captions;
mod classify;
mod transcribe;

pub use captions::*;
pub use classify::*;
pub use transcribe::*;

pub struct AudioPipelines<'a> {
    _signals: std::sync::Arc<nim_signals::Client>,
    _services: nim_state::Services<'a>,
}

impl<'a> AudioPipelines<'a> {
    pub fn new(pool: &'a nim_store::PgPool, signals: std::sync::Arc<nim_signals::Client>) -> Self {
        Self {
            _signals: signals,
            _services: nim_state::Services::new(pool),
        }
    }
}
