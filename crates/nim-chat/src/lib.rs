mod annotation;
mod content;
mod media;
mod message;

pub use annotation::*;
pub use content::*;
pub use media::*;
pub use message::*;

pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Parse<T>: Sized {
    type Error;
    type Context;

    fn parse(&self, ctx: &Self::Context) -> BoxFuture<'_, Result<T, Self::Error>>;
}

pub trait Render<T> {
    type Error;
    type Output;
    type Context;

    fn render(&self, ctx: &Self::Context, v: T) -> BoxFuture<'_, Result<Self::Output, Self::Error>>;
}
