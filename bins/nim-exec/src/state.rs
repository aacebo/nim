pub struct State<'a> {
    pool: &'a sqlx::PgPool,
}

impl<'a> State<'a> {
    pub fn new(pool: &'a sqlx::PgPool) -> Self {
        Self { pool }
    }
}
