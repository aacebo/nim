pub struct SearchItem<T> {
    pub similarity: f32,

    inner: T,
}

impl<T> SearchItem<T> {
    pub fn new(similarity: f32, inner: T) -> Self {
        Self { similarity, inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'r, T> sqlx::FromRow<'r, sqlx::postgres::PgRow> for SearchItem<T>
where
    T: sqlx::FromRow<'r, sqlx::postgres::PgRow>,
{
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        Ok(Self {
            similarity: row.try_get("similarity")?,
            inner: T::from_row(row)?,
        })
    }
}

impl<T> std::ops::Deref for SearchItem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for SearchItem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
