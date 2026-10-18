use crate::Version;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Prediction {
    pub id: PredictionId,
    pub version: Version,
    pub hypothesis: String,
    pub confidence: f32,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PredictionId(uuid::Uuid);

impl PredictionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for PredictionId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<PredictionId> for uuid::Uuid {
    fn from(value: PredictionId) -> Self {
        value.0
    }
}

impl std::fmt::Display for PredictionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pred_{}", self.0.simple())
    }
}

impl serde::Serialize for PredictionId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for PredictionId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("pred_")
            .ok_or_else(|| serde::de::Error::custom("invalid prediction id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

impl From<nim_store::PredictionRow> for Prediction {
    fn from(value: nim_store::PredictionRow) -> Self {
        Self {
            id: value.id.into(),
            version: value.version.into(),
            hypothesis: value.hypothesis,
            confidence: value.confidence as f32,
            deadline: value.deadline,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
