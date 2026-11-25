pub mod chatml;
mod cursor;
mod parser;

pub use cursor::*;
pub use parser::*;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
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
