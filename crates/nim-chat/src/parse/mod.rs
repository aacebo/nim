#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum ParseMode {
    /// Reject unsupported fields and messages.
    #[default]
    Strict,

    /// Ignore unsupported fields and messages.
    Lenient,
}

#[derive(Debug, Copy, Clone)]
pub struct Parser<'a> {
    pub mode: ParseMode,
    pub src: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a str, mode: ParseMode) -> Self {
        Self { src, mode }
    }

    pub fn mode(&self) -> &ParseMode {
        &self.mode
    }

    pub fn take_prefix(&mut self, prefix: &str) -> bool {
        match self.src.strip_prefix(prefix) {
            Some(remaining) => {
                self.src = remaining;
                true
            }
            None => false,
        }
    }

    pub fn take_until(&mut self, delimiter: &str) -> Option<&'a str> {
        let index = self.src.find(delimiter)?;
        let value = &self.src[..index];
        self.src = &self.src[index + delimiter.len()..];
        Some(value)
    }

    pub fn take_remaining(&mut self) -> &'a str {
        std::mem::take(&mut self.src)
    }
}
