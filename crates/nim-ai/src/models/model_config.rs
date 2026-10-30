use crate::models::{Architecture, ModelType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelConfig {
    #[serde(default)]
    pub model_type: ModelType,

    #[serde(default)]
    pub architectures: Vec<Architecture>,

    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

impl ModelConfig {
    pub fn is_token_classification(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture.as_str().ends_with("ForTokenClassification"))
    }

    pub fn is_sequence_classification(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture.as_str().ends_with("ForSequenceClassification"))
    }

    #[cfg(feature = "audio")]
    pub fn is_audio_captioning(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture.as_str() == "WhisperForAudioCaptioning")
    }

    #[cfg(feature = "audio")]
    pub fn is_cnn14_audio_captioning(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture.as_str() == "Cnn14RnnTempAttnGruModel")
    }

    #[cfg(feature = "audio")]
    pub fn is_cnn8rnn_audio_classification(&self) -> bool {
        self.architectures
            .iter()
            .any(|architecture| architecture.as_str() == "Cnn8RnnSoundEventDetection")
    }
}
