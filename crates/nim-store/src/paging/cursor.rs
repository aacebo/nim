#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Cursor {
    pub id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut bytes = [0; 28];
        bytes[..16].copy_from_slice(self.id.as_bytes());
        bytes[16..24].copy_from_slice(&self.timestamp.timestamp().to_be_bytes());
        bytes[24..].copy_from_slice(&self.timestamp.timestamp_subsec_nanos().to_be_bytes());
        f.write_str(&hex::encode(bytes))
    }
}

impl std::str::FromStr for Cursor {
    type Err = ParseCursorError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; 28];
        hex::decode_to_slice(value, &mut bytes).map_err(|_| ParseCursorError)?;

        let id = uuid::Uuid::from_bytes(bytes[..16].try_into().map_err(|_| ParseCursorError)?);
        let seconds = i64::from_be_bytes(bytes[16..24].try_into().map_err(|_| ParseCursorError)?);
        let nanos = u32::from_be_bytes(bytes[24..].try_into().map_err(|_| ParseCursorError)?);
        let timestamp = chrono::DateTime::from_timestamp(seconds, nanos).ok_or(ParseCursorError)?;

        Ok(Self { id, timestamp })
    }
}

impl serde::Serialize for Cursor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for Cursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct ParseCursorError;

impl std::fmt::Display for ParseCursorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid paging cursor")
    }
}

impl std::error::Error for ParseCursorError {}
