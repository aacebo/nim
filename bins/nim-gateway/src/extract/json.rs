use axum::extract::{FromRequest, Request};

use crate::Error;

#[derive(Debug)]
pub struct Json<T>(T);

impl<T> Json<T> {
    #[allow(unused)]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T, S> FromRequest<S> for Json<T>
where
    T: serde::de::DeserializeOwned + serde_valid::Validate + Send,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::Json(body) = axum::Json::<T>::from_request(request, state).await?;
        body.validate()?;
        Ok(Self(body))
    }
}
