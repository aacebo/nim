mod parse;
mod render;

pub use parse::*;

use crate::{Chat, Format, Parser, render::Formatter};

pub struct ChatML;

impl Format for ChatML {
    type Error = nim_error::Diagnostic;

    fn parse(&self, src: &str) -> Result<Chat, Self::Error> {
        let mut parser = Parser::from_src(src);
        parse::parse_chat(&mut parser)
    }

    fn render(&self, chat: &Chat) -> Result<String, Self::Error> {
        let mut output = Vec::new();
        let mut formatter = Formatter::new(&mut output);
        render::render_chat(&mut formatter, chat)?;
        String::from_utf8(output).map_err(nim_error::IntoDiagnostic::into_diagnostic)
    }
}
