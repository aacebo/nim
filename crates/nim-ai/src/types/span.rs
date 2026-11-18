#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Span {
    Offset(Offset),
    Space(BoundingBox),
    Time(TimeRange),
}

impl Span {
    pub fn as_offset(&self) -> Option<&Offset> {
        match self {
            Self::Offset(offset) => Some(offset),
            Self::Space(_) | Self::Time(_) => None,
        }
    }

    pub fn as_space(&self) -> Option<&BoundingBox> {
        match self {
            Self::Space(v) => Some(v),
            Self::Offset(_) | Self::Time(_) => None,
        }
    }

    pub fn as_time(&self) -> Option<&TimeRange> {
        match self {
            Self::Time(v) => Some(v),
            Self::Offset(_) | Self::Space(_) => None,
        }
    }

    pub fn text<'a>(&self, input: &'a [u8]) -> nim_error::Result<&'a str> {
        let Self::Offset(offset) = self else {
            return Err(nim_error::Diagnostic::message("Span::text called on non-offset span"));
        };

        let text = std::str::from_utf8(input)?;
        let boundary = |index: u32| {
            text.char_indices()
                .map(|(index, _)| index)
                .chain(std::iter::once(text.len()))
                .nth(index as usize)
        };

        let start = boundary(offset.begin);
        let end = boundary(offset.end);

        match (start, end) {
            (Some(start), Some(end)) if start <= end => Ok(&text[start..end]),
            _ => Err(nim_error::Diagnostic::message("text span is outside the input")),
        }
    }
}

impl From<Offset> for Span {
    fn from(value: Offset) -> Self {
        Self::Offset(value)
    }
}

impl From<BoundingBox> for Span {
    fn from(value: BoundingBox) -> Self {
        Self::Space(value)
    }
}

impl From<TimeRange> for Span {
    fn from(value: TimeRange) -> Self {
        Self::Time(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Offset {
    pub begin: u32,
    pub end: u32,
}

impl Offset {
    pub fn new(begin: u32, end: u32) -> Self {
        Self { begin, end }
    }
}

impl std::fmt::Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.begin, self.end)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimeRange {
    pub begin: f64,
    pub end: f64,
}

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox {
    pub xmin: f32,
    pub ymin: f32,
    pub xmax: f32,
    pub ymax: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_span_uses_character_offsets() {
        let span = Span::Offset(Offset::new(1, 3));
        assert_eq!(span.text("aé🙂z".as_bytes()).unwrap(), "é🙂");
        assert!(Span::Offset(Offset::new(3, 2)).text(b"abc").is_err());
        assert!(Span::Offset(Offset::new(0, 5)).text(b"abc").is_err());
        assert!(Span::Time(TimeRange { begin: 0.0, end: 1.0 }).text(b"abc").is_err());
    }

    #[test]
    fn span_variants_round_trip() {
        let spans = [
            Span::Offset(Offset::new(0, 2)),
            Span::Space(BoundingBox {
                xmin: 1.0,
                ymin: 2.0,
                xmax: 3.0,
                ymax: 4.0,
            }),
            Span::Time(TimeRange { begin: 0.0, end: 1.0 }),
        ];
        for span in spans {
            let json = serde_json::to_string(&span).unwrap();
            assert_eq!(serde_json::from_str::<Span>(&json).unwrap(), span);
        }
    }
}
