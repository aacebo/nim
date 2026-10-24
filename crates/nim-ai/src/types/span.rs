use crate::Error;

use super::{BoundingBox, Offset};

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Span {
    Text(Offset),
    Image(BoundingBox),
}

impl Span {
    pub fn as_text(&self) -> Option<&Offset> {
        match self {
            Self::Text(offset) => Some(offset),
            Self::Image(_) => None,
        }
    }

    pub fn text<'a>(&self, input: &'a [u8]) -> Result<Option<&'a str>, Error> {
        let Self::Text(offset) = self else {
            return Ok(None);
        };
        let text = std::str::from_utf8(input).map_err(|err| Error::custom("ai::encoding", err))?;
        let boundary = |index: u32| {
            text.char_indices()
                .map(|(index, _)| index)
                .chain(std::iter::once(text.len()))
                .nth(index as usize)
        };
        let start = boundary(offset.begin);
        let end = boundary(offset.end);

        match (start, end) {
            (Some(start), Some(end)) if start <= end => Ok(Some(&text[start..end])),
            _ => Err(Error::custom("ai::invalid", "text span is outside the input")),
        }
    }
}
