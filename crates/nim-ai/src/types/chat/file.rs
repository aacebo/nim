#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    Txt,
    Jpg,
    Png,
    Wav,
    Mp3,
}

impl FileFormat {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Txt => "txt",
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
        }
    }
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
