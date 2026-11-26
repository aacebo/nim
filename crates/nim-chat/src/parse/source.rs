use std::path::PathBuf;

use crate::parse::{Location, Span};

/// Primarily used to map spans (0 based byte index ranges)
/// to source text.
/// Source text with a stable identifier for span resolution.
#[derive(Debug, Clone)]
pub struct Source<'a> {
    /// the filepath if this source belongs to a file
    /// on disk.
    path: Option<PathBuf>,

    /// raw source text
    text: &'a str,

    /// file-wide byte range
    span: Span,

    /// line start offsets, in bytes and characters
    lines: Vec<(usize, usize)>,
}

impl<'a> Source<'a> {
    pub(crate) fn new(start: usize, text: &'a str) -> Self {
        let mut lines = vec![(0, 0)];

        for (offset, (index, ch)) in text.char_indices().enumerate() {
            if ch == '\n' {
                lines.push((index + ch.len_utf8(), offset + 1));
            }
        }

        Self {
            path: None,
            text,
            span: Span::new(start, start + text.len()),
            lines,
        }
    }

    pub fn from_src(src: &'a str) -> Self {
        Self::new(0, src)
    }

    pub fn from_file(path: impl Into<PathBuf>, src: &'a str) -> Self {
        let path = path.into();
        let mut src = Self::new(0, src);
        src.path = Some(path);
        src
    }

    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    pub fn text(&self) -> &str {
        self.text
    }

    pub fn span(&self) -> Span {
        self.span
    }

    /// Resolves the given span into a byte index range.
    pub fn range(&self, span: Span) -> std::ops::Range<usize> {
        let r = span.byte_range();
        self.byte(r.start)..self.byte(r.end)
    }

    /// Gets a sub span of source text from the file.
    pub fn slice(&self, span: Span) -> &str {
        &self.text[self.range(span)]
    }

    /// Resolves a global byte index within this file into a 0-based `Location`.
    pub fn location(&self, i: usize) -> Location {
        let index = i - self.span.byte_range().start;

        let line = self
            .lines
            .binary_search_by_key(&index, |(offset, _)| *offset)
            .unwrap_or_else(|next| next - 1);
        let (start, offset) = self.lines[line];
        let column = self.text[start..]
            .char_indices()
            .take_while(|(byte, _)| *byte < index - start)
            .count();
        Location {
            offset: offset + column,
            line,
            column,
        }
    }

    /// Returns the source-relative byte index for a global byte index.
    pub fn byte(&self, i: usize) -> usize {
        (i - self.span.byte_range().start).min(self.text.len())
    }
}
