#![allow(dead_code)]

#[derive(Clone)]
pub struct State<'a> {
    pub pool: std::sync::Arc<sqlx::PgPool>,
    pub services: nim::state::Services<'a>,
}

impl<'a> State<'a> {
    pub fn new(pool: std::sync::Arc<sqlx::PgPool>) -> Self {
        Self {
            pool,
            services: nim::state::Services::new(pool.as_ref()),
        }
    }
}
