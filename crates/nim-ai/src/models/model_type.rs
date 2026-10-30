use crate::Error;

#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Bart,
    Bert,
    DistilBert,
    Roberta,
    Deberta,
    Gpt2,
    Llama,
    Mistral,
    T5,
    #[serde(rename = "mobilenet_v2")]
    MobileNetV2,
    Detr,
    Git,
    Whisper,
    #[default]
    #[serde(other)]
    Unknown,
}

impl ModelType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bart => "bart",
            Self::Bert => "bert",
            Self::DistilBert => "distilbert",
            Self::Roberta => "roberta",
            Self::Deberta => "deberta",
            Self::Gpt2 => "gpt2",
            Self::Llama => "llama",
            Self::Mistral => "mistral",
            Self::T5 => "t5",
            Self::MobileNetV2 => "mobilenet_v2",
            Self::Detr => "detr",
            Self::Git => "git",
            Self::Whisper => "whisper",
            Self::Unknown => "??",
        }
    }
}

impl std::str::FromStr for ModelType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Error> {
        Ok(match value.to_lowercase().as_str() {
            "bart" => Self::Bart,
            "bert" => Self::Bert,
            "distilbert" => Self::DistilBert,
            "roberta" => Self::Roberta,
            "deberta" => Self::Deberta,
            "gpt2" => Self::Gpt2,
            "llama" => Self::Llama,
            "mistral" => Self::Mistral,
            "t5" => Self::T5,
            "mobilenet_v2" => Self::MobileNetV2,
            "detr" => Self::Detr,
            "git" => Self::Git,
            "whisper" => Self::Whisper,
            _ => Self::Unknown,
        })
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
