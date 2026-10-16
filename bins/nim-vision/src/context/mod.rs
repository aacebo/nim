pub mod http;

#[derive(Clone)]
pub struct Context {
    pool: std::sync::Arc<sqlx::PgPool>,
}

impl Context {
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self { pool }
    }

    pub fn services(&self) -> nim::state::Services<'_> {
        nim::state::Services::new(&self.pool)
    }
}
