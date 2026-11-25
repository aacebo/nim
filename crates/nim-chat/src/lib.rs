mod annotation;
mod content;
mod event;
mod media;
mod message;
mod parser;
mod tool;
pub mod widgets;

pub use annotation::*;
pub use content::*;
pub use event::*;
pub use media::*;
pub use message::*;
pub use parser::*;
pub use tool::*;

pub trait Parse<T>: Sized {
    type Error;
    type Context<'a>
    where
        Self: 'a;

    fn parse<'a>(&'a self, ctx: &'a Self::Context<'a>) -> futures_util::future::BoxFuture<'a, Result<T, Self::Error>>;
}

pub trait Render<T> {
    type Error;
    type Output;
    type Context<'a>
    where
        Self: 'a;

    fn render<'a>(
        &'a self,
        ctx: &'a Self::Context<'a>,
        v: T,
    ) -> futures_util::future::BoxFuture<'a, Result<Self::Output, Self::Error>>;
}
