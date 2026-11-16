use std::ops::{Bound, RangeBounds};

#[derive(Debug)]
pub struct TimeRange {
    pub start: Bound<chrono::DateTime<chrono::Utc>>,
    pub end: Bound<chrono::DateTime<chrono::Utc>>,
}

impl Default for TimeRange {
    fn default() -> Self {
        Self {
            start: Bound::Unbounded,
            end: Bound::Unbounded,
        }
    }
}

impl RangeBounds<chrono::DateTime<chrono::Utc>> for TimeRange {
    fn start_bound(&self) -> Bound<&chrono::DateTime<chrono::Utc>> {
        self.start.as_ref()
    }

    fn end_bound(&self) -> Bound<&chrono::DateTime<chrono::Utc>> {
        self.end.as_ref()
    }
}

impl sqlx::Type<sqlx::Postgres> for TimeRange {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <sqlx::postgres::types::PgRange<chrono::DateTime<chrono::Utc>> as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        <sqlx::postgres::types::PgRange<chrono::DateTime<chrono::Utc>> as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for TimeRange {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let range = sqlx::postgres::types::PgRange {
            start: self.start,
            end: self.end,
        };

        sqlx::Encode::encode_by_ref(&range, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TimeRange {
    fn decode(value: <sqlx::Postgres as sqlx::Database>::ValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let range =
            <sqlx::postgres::types::PgRange<chrono::DateTime<chrono::Utc>> as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;

        Ok(Self {
            start: range.start,
            end: range.end,
        })
    }
}
