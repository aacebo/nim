use axum::response::IntoResponse;

use crate::{Error, http::HttpContext};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, serde_valid::Validate)]
pub struct Query {
    pub cursor: Option<nim::state::Cursor>,

    #[validate(minimum = 5)]
    #[validate(maximum = 500)]
    #[serde(default = "defaults::limit")]
    pub limit: usize,
}

pub async fn route(ctx: HttpContext, query: axum::extract::Query<Query>) -> axum::response::Result<axum::response::Response> {
    serde_valid::Validate::validate(&query.0).map_err(Error::from)?;
    let items = ctx
        .services()
        .memories()
        .find(nim::state::Page {
            cursor: query.cursor,
            limit: query.limit,
        })
        .await
        .map_err(Error::from)?;

    Ok(axum::extract::Json(items).into_response())
}

mod defaults {
    pub fn limit() -> usize {
        5
    }
}
