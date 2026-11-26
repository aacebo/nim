use crate::{
    Cursor, ParseMode,
    parse::{Source, Span},
};

pub const EOF: char = '\x04';

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tag {
    Json,
    Think,
    Widget,
}

impl Tag {
    pub fn open(self) -> &'static str {
        match self {
            Self::Json => "<json>",
            Self::Think => "<think>",
            Self::Widget => "<widget>",
        }
    }

    pub fn close(self) -> &'static str {
        match self {
            Self::Json => "</json>",
            Self::Think => "</think>",
            Self::Widget => "</widget>",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    mode: ParseMode,
    cursor: Cursor<'a>,
    source: Source<'a>,
    stack: Vec<Tag>,
}

impl<'a> Parser<'a> {
    pub fn from_src(src: &'a str, mode: ParseMode) -> Self {
        Self {
            mode,
            source: Source::from_src(src),
            cursor: Cursor::new(src),
            stack: Default::default(),
        }
    }

    pub fn mode(&self) -> &ParseMode {
        &self.mode
    }

    pub fn tag(&self) -> Option<&Tag> {
        self.stack.last()
    }

    pub fn remaining(&self) -> &'a str {
        self.cursor.src()
    }

    pub fn is_eof(&self) -> bool {
        self.cursor.is_empty()
    }

    pub fn span(&self) -> Span {
        self.cursor.span()
    }

    pub fn curr(&self) -> char {
        match self.cursor.first() {
            None => EOF,
            Some(v) => v,
        }
    }

    pub fn starts_with(&self, s: &str) -> bool {
        self.cursor.starts_with(s)
    }

    pub fn peek(&self) -> Option<char> {
        self.peek_n(1)
    }

    pub fn peek_n(&self, offset: usize) -> Option<char> {
        self.cursor.src().chars().nth(offset)
    }

    pub fn seek(&self, c: char) -> Option<usize> {
        let cursor = self.cursor.skip_while(|curr| curr != c);

        if cursor.is_empty() {
            None
        } else {
            Some(cursor.offset() as usize)
        }
    }

    #[track_caller]
    pub fn error(&self, message: impl std::fmt::Display) -> nim_error::Diagnostic {
        nim_error::Diagnostic::message(message)
            .field("mode", self.mode)
            .field("start", self.source.location(self.span().start()))
            .field("end", self.source.location(self.span().end()))
            .field("text", self.source.slice(self.cursor.span()))
    }

    pub fn advance(&mut self) -> &mut Self {
        self.cursor = self.cursor.advance();
        self
    }

    pub fn advance_by(&mut self, n: usize) -> &mut Self {
        self.cursor = self.cursor.advance_by(n);
        self
    }

    pub fn skip_while(&mut self, p: impl FnMut(char) -> bool) -> &mut Self {
        self.cursor = self.cursor.skip_while(p);
        self
    }

    pub fn parse_open(&mut self, tag: Tag) -> Result<&mut Self, nim_error::Diagnostic> {
        if self.remaining().starts_with(tag.open()) {
            self.advance_by(tag.open().len());
            self.stack.push(tag);
            Ok(self)
        } else {
            Err(self.error(format!("expected '{}'", tag.open())))
        }
    }

    pub fn parse_close(&mut self, tag: Tag) -> Result<&mut Self, nim_error::Diagnostic> {
        if self.remaining().starts_with(tag.close()) {
            let Some(_) = self.stack.pop_if(|v| tag.eq(v)) else {
                return Err(self.error(format!("invalid attempt to close a tag of type '{}'", tag.open())));
            };

            self.advance_by(tag.close().len());
            Ok(self)
        } else {
            Err(self.error(format!("expected '{}'", tag.close())))
        }
    }
}

impl<'a> Parser<'a> {
    pub fn peek_url(&self) -> Option<(usize, url::Url)> {
        let src = self.remaining();

        if !src.starts_with("http://") && !src.starts_with("https://") {
            return None;
        }

        let mut end = src.find(|c: char| c.is_whitespace() || c == '<').unwrap_or(src.len());

        if let Some((index, _)) = src[..end].match_indices(',').find(|(index, _)| {
            let rest = &src[index + 1..];
            rest.starts_with("http://") || rest.starts_with("https://")
        }) {
            end = index;
        }

        let value = src[..end].trim_end_matches(['.', ',', ';', ':', '!', '?', ')', '}', ']']);
        let url = url::Url::parse(value).ok()?;
        Some((value.len(), url))
    }

    pub fn peek_json(&self) -> Option<(usize, serde_json::Value)> {
        let src = self.remaining();

        if !src.starts_with('{') && !src.starts_with('[') {
            return None;
        }

        let mut values = serde_json::Deserializer::from_str(src).into_iter::<serde_json::Value>();
        let value = values.next()?.ok()?;
        Some((values.byte_offset(), value))
    }

    pub fn parse_json(&mut self) -> Result<serde_json::Value, nim_error::Diagnostic> {
        let src = self.remaining();

        if !src.starts_with('{') && !src.starts_with('[') {
            return Err(self.error("expected json object opening '{' or '['"));
        }

        let mut values = serde_json::Deserializer::from_str(src).into_iter::<serde_json::Value>();
        let value = values.next().ok_or(self.error("invalid json"))?;
        let value = value?;
        self.advance_by(values.byte_offset());
        Ok(value)
    }

    pub fn seek_code_end(&self, offset: usize) -> Option<usize> {
        let base = self.cursor.offset() as usize;
        let index = offset.checked_sub(base)?;
        let src = self.remaining();
        let rest = src.get(index..)?;
        let marker = rest.chars().next()?;

        if marker != '`' && marker != '~' {
            return None;
        }

        let count = rest.chars().take_while(|c| *c == marker).count();
        let line_start = src[..index].rfind('\n').map_or(0, |i| i + 1);

        if count >= 3 && src[line_start..index].trim().is_empty() {
            let mut offset = rest.find('\n').map_or(src.len(), |i| index + i + 1);

            while offset < src.len() {
                let end = src[offset..].find('\n').map_or(src.len(), |i| offset + i);
                let line = src[offset..end].trim_end_matches('\r');
                let trimmed = line.trim_start_matches([' ', '\t']);
                let indent = line.len() - trimmed.len();
                let closing = trimmed.chars().take_while(|c| *c == marker).count();

                if indent <= 3 && closing >= count && trimmed[closing..].trim_matches([' ', '\t']).is_empty() {
                    return Some(base + (end + 1).min(src.len()));
                }

                offset = (end + 1).min(src.len());
            }

            return Some(base + src.len());
        }

        if marker == '`' {
            let delimiter = &rest[..count];
            let end = rest[count..].find(delimiter).map_or(src.len(), |i| index + count + i + count);
            return Some(base + end);
        }

        None
    }
}
