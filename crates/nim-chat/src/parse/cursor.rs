/// Zero-copy immutable cursor over source text.
/// Each parse step returns a new advanced cursor.
/// A copyable cursor over source text during lexing.
#[derive(Debug, Copy, Clone)]
pub struct Cursor<'a> {
    src: &'a str,
    offset: u32,
}

impl<'a> Cursor<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, offset: 0 }
    }

    pub fn src(self) -> &'a str {
        self.src
    }

    pub fn offset(self) -> u32 {
        self.offset
    }

    pub fn is_empty(self) -> bool {
        self.src.is_empty()
    }

    pub fn first(self) -> Option<char> {
        self.src.chars().next()
    }

    pub fn starts_with(self, s: &str) -> bool {
        self.src.starts_with(s)
    }

    /// Get a slice of text from the current cursor to the provided end.
    pub fn slice_to(self, end: Cursor<'_>) -> &'a str {
        let len = (end.offset - self.offset) as usize;
        &self.src[..len]
    }

    pub fn advance(self) -> Self {
        self.advance_by(self.first().map_or(0, char::len_utf8))
    }

    pub fn advance_by(self, n: usize) -> Self {
        Self {
            src: &self.src[n..],
            offset: self.offset + n as u32,
        }
    }

    /// Advance while predicate holds on chars.
    pub fn skip_while(self, mut pred: impl FnMut(char) -> bool) -> Self {
        let mut bytes = 0;

        for ch in self.src.chars() {
            if !pred(ch) {
                break;
            }

            bytes += ch.len_utf8();
        }

        self.advance_by(bytes)
    }
}
