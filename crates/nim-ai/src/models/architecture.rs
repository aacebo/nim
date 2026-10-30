#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Architecture {
    BertModel,
    BertForTokenClassification,
    BertForSequenceClassification,
    DistilBertModel,
    DistilBertForTokenClassification,
    DistilBertForSequenceClassification,
    BartModel,
    BartForConditionalGeneration,
    MobileNetV2ForImageClassification,
    DetrForObjectDetection,
    GitForCausalLM,
    WhisperForConditionalGeneration,
    WhisperForAudioCaptioning,
    Cnn14RnnTempAttnGruModel,
    Cnn8RnnSoundEventDetection,
    Other(String),
}

impl Architecture {
    pub fn as_str(&self) -> &str {
        match self {
            Self::BertModel => "BertModel",
            Self::BertForTokenClassification => "BertForTokenClassification",
            Self::BertForSequenceClassification => "BertForSequenceClassification",
            Self::DistilBertModel => "DistilBertModel",
            Self::DistilBertForTokenClassification => "DistilBertForTokenClassification",
            Self::DistilBertForSequenceClassification => "DistilBertForSequenceClassification",
            Self::BartModel => "BartModel",
            Self::BartForConditionalGeneration => "BartForConditionalGeneration",
            Self::MobileNetV2ForImageClassification => "MobileNetV2ForImageClassification",
            Self::DetrForObjectDetection => "DetrForObjectDetection",
            Self::GitForCausalLM => "GitForCausalLM",
            Self::WhisperForConditionalGeneration => "WhisperForConditionalGeneration",
            Self::WhisperForAudioCaptioning => "WhisperForAudioCaptioning",
            Self::Cnn14RnnTempAttnGruModel => "Cnn14RnnTempAttnGruModel",
            Self::Cnn8RnnSoundEventDetection => "Cnn8RnnSoundEventDetection",
            Self::Other(value) => value,
        }
    }
}

impl std::str::FromStr for Architecture {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "BertModel" => Self::BertModel,
            "BertForTokenClassification" => Self::BertForTokenClassification,
            "BertForSequenceClassification" => Self::BertForSequenceClassification,
            "DistilBertModel" => Self::DistilBertModel,
            "DistilBertForTokenClassification" => Self::DistilBertForTokenClassification,
            "DistilBertForSequenceClassification" => Self::DistilBertForSequenceClassification,
            "BartModel" => Self::BartModel,
            "BartForConditionalGeneration" => Self::BartForConditionalGeneration,
            "MobileNetV2ForImageClassification" => Self::MobileNetV2ForImageClassification,
            "DetrForObjectDetection" => Self::DetrForObjectDetection,
            "GitForCausalLM" => Self::GitForCausalLM,
            "WhisperForConditionalGeneration" => Self::WhisperForConditionalGeneration,
            "WhisperForAudioCaptioning" => Self::WhisperForAudioCaptioning,
            "Cnn14RnnTempAttnGruModel" => Self::Cnn14RnnTempAttnGruModel,
            "Cnn8RnnSoundEventDetection" => Self::Cnn8RnnSoundEventDetection,
            _ => Self::Other(value.to_string()),
        })
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl serde::Serialize for Architecture {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Architecture {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}
