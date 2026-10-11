#![allow(dead_code)]

#[derive(Clone)]
pub struct State {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl State {
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }

    pub fn services(&self) -> nim::state::Services<'_> {
        nim::state::Services::new(&self.pool)
    }
}
