use crate::{EntityId, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub parent_id: Option<GoalId>,
    pub version: Version,
    pub description: String,
    pub status: GoalStatus,
    pub priority: f32,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub conditions: Vec<GoalCondition>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoalId(uuid::Uuid);

impl GoalId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for GoalId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<GoalId> for uuid::Uuid {
    fn from(value: GoalId) -> Self {
        value.0
    }
}

impl std::fmt::Display for GoalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "goal_{}", self.0.simple())
    }
}

impl serde::Serialize for GoalId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for GoalId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("goal_")
            .ok_or_else(|| serde::de::Error::custom("invalid goal id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    Active,
    Suspended,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for GoalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<nim_store::GoalRow> for Goal {
    fn from(value: nim_store::GoalRow) -> Self {
        Self {
            id: value.id.into(),
            parent_id: value.parent_id.map(Into::into),
            version: value.version.into(),
            description: value.description,
            status: value.status.into(),
            priority: value.priority as f32,
            deadline: value.deadline,
            conditions: value.conditions.into_inner().into_iter().map(Into::into).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<nim_store::GoalStatus> for GoalStatus {
    fn from(value: nim_store::GoalStatus) -> Self {
        match value {
            nim_store::GoalStatus::Active => Self::Active,
            nim_store::GoalStatus::Suspended => Self::Suspended,
            nim_store::GoalStatus::Completed => Self::Completed,
            nim_store::GoalStatus::Failed => Self::Failed,
            nim_store::GoalStatus::Cancelled => Self::Cancelled,
            _ => unreachable!(),
        }
    }
}

impl From<GoalStatus> for nim_store::GoalStatus {
    fn from(value: GoalStatus) -> Self {
        match value {
            GoalStatus::Active => Self::Active,
            GoalStatus::Suspended => Self::Suspended,
            GoalStatus::Completed => Self::Completed,
            GoalStatus::Failed => Self::Failed,
            GoalStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GoalCondition {
    EntityState { entity: EntityId, state: String },
    Fact { description: String },
    Time { before: chrono::DateTime<chrono::Utc> },
    Custom { description: String },
}

impl From<nim_store::GoalCondition> for GoalCondition {
    fn from(value: nim_store::GoalCondition) -> Self {
        match value {
            nim_store::GoalCondition::EntityState { entity, state } => Self::EntityState {
                entity: entity.into(),
                state,
            },
            nim_store::GoalCondition::Fact { description } => Self::Fact { description },
            nim_store::GoalCondition::Time { before } => Self::Time { before },
            nim_store::GoalCondition::Custom { description } => Self::Custom { description },
            _ => unreachable!(),
        }
    }
}

impl From<GoalCondition> for nim_store::GoalCondition {
    fn from(value: GoalCondition) -> Self {
        match value {
            GoalCondition::EntityState { entity, state } => Self::EntityState {
                entity: entity.into(),
                state,
            },
            GoalCondition::Fact { description } => Self::Fact { description },
            GoalCondition::Time { before } => Self::Time { before },
            GoalCondition::Custom { description } => Self::Custom { description },
        }
    }
}
