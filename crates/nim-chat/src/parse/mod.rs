pub mod chatml;
mod cursor;
mod parser;
mod source;
mod span;

pub use cursor::*;
pub use parser::*;
pub use source::*;
pub use span::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, serde::Serialize)]
pub enum ParseMode {
    /// Reject unsupported fields and messages.
    #[default]
    Strict,

    /// Ignore unsupported fields and messages.
    Lenient,
}

pub trait Parse: Sized {
    type Error;

    fn parse(p: &mut Parser) -> Result<Self, Self::Error>;
}
