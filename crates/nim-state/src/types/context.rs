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

impl From<nim_store::Context> for Context {
    fn from(value: nim_store::Context) -> Self {
        match value {
            nim_store::Context::Temporal { start, end } => Self::Temporal { start, end },
            nim_store::Context::Spatial {
                place,
                latitude,
                longitude,
            } => Self::Spatial {
                place,
                latitude,
                longitude,
            },
            nim_store::Context::Social { entities } => Self::Social {
                entities: entities.into_iter().map(EntityId::from).collect(),
            },
            nim_store::Context::Emotional { valence, arousal } => Self::Emotional { valence, arousal },
            nim_store::Context::Goal { goals } => Self::Goal {
                goals: goals.into_iter().map(GoalId::from).collect(),
            },
            nim_store::Context::Situational { activity, environment } => Self::Situational { activity, environment },
            _ => unreachable!(),
        }
    }
}
