use crate::State;

#[tracing::instrument(name = "upload", skip(state))]
pub async fn route(state: State) -> axum::response::Response {
    todo!()
}
