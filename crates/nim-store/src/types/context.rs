#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Context {
    Temporal {
        start: chrono::DateTime<chrono::Utc>,
        end: Option<chrono::DateTime<chrono::Utc>>,
    },
    Spatial {
        place: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
    },
    Social {
        entities: Vec<uuid::Uuid>,
    },
    Emotional {
        valence: f32,
        arousal: f32,
    },
    Goal {
        goals: Vec<uuid::Uuid>,
    },
    Situational {
        activity: Option<String>,
        environment: Option<String>,
    },
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Region {
    Visual,
    Audio,
    Memory,
    Emotion,
    Executive,
    Action,
}
