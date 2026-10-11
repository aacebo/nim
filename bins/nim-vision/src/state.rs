use axum::extract::FromRequestParts;

use crate::Error;

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

impl<S> FromRequestParts<S> for State
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts.extensions.get::<Self>().cloned().expect("state not present"))
    }
}
