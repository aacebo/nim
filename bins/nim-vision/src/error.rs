#[derive(Debug)]
pub enum Error {
    Core(nim::Error),
    Http(axum::response::Response),
}

impl From<nim::Error> for Error {
    fn from(value: nim::Error) -> Self {
        Self::Core(value)
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Http(err) => err,
            Self::Core(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
