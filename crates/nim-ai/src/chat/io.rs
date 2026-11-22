use super::CompatMode;

pub struct Parser<'a> {
    remaining: &'a str,
    compat: CompatMode,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str, compat: CompatMode) -> Self {
        Self {
            remaining: text.trim(),
            compat,
        }
    }

    pub fn compat(&self) -> CompatMode {
        self.compat
    }

    pub fn take_prefix(&mut self, prefix: &str) -> bool {
        match self.remaining.strip_prefix(prefix) {
            Some(remaining) => {
                self.remaining = remaining;
                true
            }
            None => false,
        }
    }

    pub fn take_until(&mut self, delimiter: &str) -> Option<&'a str> {
        let index = self.remaining.find(delimiter)?;
        let value = &self.remaining[..index];
        self.remaining = &self.remaining[index + delimiter.len()..];

        Some(value)
    }

    pub fn take_remaining(&mut self) -> &'a str {
        std::mem::take(&mut self.remaining)
    }
}
