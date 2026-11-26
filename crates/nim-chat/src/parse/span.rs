/// 0 indexed char based location
/// A char, line, and column location within a source file.
#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Location {
    /// char offset
    pub offset: usize,

    /// line number
    pub line: usize,

    /// column number
    pub column: usize,
}

/// A source range.
#[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Span {
    /// the start byte index (inclusive).
    start: usize,

    /// the end byte index (exclusive).
    end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub const fn byte_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    pub const fn start(&self) -> usize {
        self.start
    }

    pub const fn end(&self) -> usize {
        self.end
    }

    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn contains(&self, i: usize) -> bool {
        i >= self.start && i < self.end
    }

    pub const fn is_subset(&self, other: &Self) -> bool {
        self.start >= other.start && self.end <= other.end
    }

    pub const fn join(self, other: Self) -> Self {
        let start = if self.start < other.start { self.start } else { other.start };
        let end = if self.end > other.end { self.end } else { other.end };
        Self { start, end }
    }
}
