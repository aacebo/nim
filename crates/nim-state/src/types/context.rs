use crate::{EntityId, GoalId};

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
        entities: Vec<EntityId>,
    },
    Emotional {
        valence: f32,
        arousal: f32,
    },
    Goal {
        goals: Vec<GoalId>,
    },
    Situational {
        activity: Option<String>,
        environment: Option<String>,
    },
}
