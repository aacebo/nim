mod annotation;
mod content;
mod event;
mod media;
mod message;
pub mod parse;
mod tool;
pub mod widgets;

pub use annotation::*;
pub use content::*;
pub use event::*;
pub use media::*;
pub use message::*;
pub use parse::{Cursor, Parse, ParseMode, Parser, Tag};
pub use tool::*;

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
