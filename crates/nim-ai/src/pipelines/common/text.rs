use crate::Error;

pub fn decode_text<'a>(input: &[&'a [u8]]) -> Result<Vec<&'a str>, Error> {
    input
        .iter()
        .map(|bytes| std::str::from_utf8(bytes).map_err(|err| Error::custom("ai::encoding", err)))
        .collect()
}
