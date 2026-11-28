mod cursor;
mod parser;
mod source;
mod span;

pub use cursor::*;
pub use parser::*;
pub use source::*;
pub use span::*;

pub trait Parse: Sized {
    type Error;

    fn parse(p: &mut Parser) -> Result<Self, Self::Error>;
}
