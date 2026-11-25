pub struct Parser<'a> {
    remaining: &'a str,
    ignore_unsupported: bool,
}

impl<'a> Parser<'a> {
    pub fn new(text: &'a str, ignore_unsupported: bool) -> Self {
        Self {
            remaining: text.trim(),
            ignore_unsupported,
        }
    }

    pub fn ignore_unsupported(&self) -> bool {
        self.ignore_unsupported
    }

    pub fn remaining(&self) -> &'a str {
        self.remaining
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
