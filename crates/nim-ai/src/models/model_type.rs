#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
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
    Qwen2,
    Qwen3,
    #[serde(rename = "gemma3_text")]
    Gemma3Text,
    T5,
    #[cfg(feature = "vision")]
    #[serde(rename = "mobilenet_v2")]
    MobileNetV2,
    #[cfg(feature = "vision")]
    Detr,
    #[cfg(feature = "vision")]
    Git,
    #[cfg(feature = "audio")]
    Whisper,
    #[cfg(feature = "audio")]
    #[serde(rename = "parler_tts")]
    ParlerTTS,
    #[default]
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
            Self::Qwen2 => "qwen2",
            Self::Qwen3 => "qwen3",
            Self::Gemma3Text => "gemma3_text",
            Self::T5 => "t5",
            #[cfg(feature = "vision")]
            Self::MobileNetV2 => "mobilenet_v2",
            #[cfg(feature = "vision")]
            Self::Detr => "detr",
            #[cfg(feature = "vision")]
            Self::Git => "git",
            #[cfg(feature = "audio")]
            Self::Whisper => "whisper",
            #[cfg(feature = "audio")]
            Self::ParlerTTS => "parler_tts",
            Self::Unknown => "??",
        }
    }
}

impl std::str::FromStr for ModelType {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> nim_error::Result<Self> {
        let value = value.to_lowercase();

        let model = match value.as_str() {
            "bart" => Self::Bart,
            "bert" => Self::Bert,
            "distilbert" => Self::DistilBert,
            "roberta" => Self::Roberta,
            "deberta" => Self::Deberta,
            "gpt2" => Self::Gpt2,
            "llama" => Self::Llama,
            "mistral" => Self::Mistral,
            "qwen2" => Self::Qwen2,
            "qwen3" => Self::Qwen3,
            "gemma3_text" => Self::Gemma3Text,
            "t5" => Self::T5,
            #[cfg(feature = "vision")]
            "mobilenet_v2" => Self::MobileNetV2,
            #[cfg(feature = "vision")]
            "detr" => Self::Detr,
            #[cfg(feature = "vision")]
            "git" => Self::Git,
            #[cfg(feature = "audio")]
            "whisper" => Self::Whisper,
            #[cfg(feature = "audio")]
            "parler_tts" => Self::ParlerTTS,
            _ => Self::Unknown,
        };

        if model == Self::Unknown && matches!(value.as_str(), "mobilenet_v2" | "detr" | "git" | "whisper" | "parler_tts") {
            return Err(nim_error::Diagnostic::message(format!(
                "model type `{value}` requires an unavailable feature"
            )));
        }

        Ok(model)
    }
}

impl<'de> serde::Deserialize<'de> for ModelType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        let model = value.parse().map_err(serde::de::Error::custom)?;

        if value != value.to_lowercase() {
            return Ok(Self::Unknown);
        }

        Ok(model)
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
