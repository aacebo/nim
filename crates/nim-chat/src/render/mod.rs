mod format;

pub use format::*;

pub trait Render {
    type Error;

    fn render(&self, f: &mut Formatter) -> Result<(), Self::Error>;
}
