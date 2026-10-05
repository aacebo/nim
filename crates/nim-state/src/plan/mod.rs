mod action;
mod condition;
mod status;

pub use action::*;
pub use condition::*;
pub use status::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub title: String,
    pub status: PlanStatus,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlanId(uuid::Uuid);

impl PlanId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl std::fmt::Display for PlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "plan_{}", self.0)
    }
}

impl serde::Serialize for PlanId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for PlanId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("plan_")
            .ok_or_else(|| serde::de::Error::custom("invalid plan id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Step {
    pub id: StepId,
    pub plan_id: PlanId,
    pub position: usize,
    pub name: String,
    pub status: StepStatus,
    pub about: Option<String>,
    pub action: Action,
    pub condition: Option<Condition>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StepId(uuid::Uuid);

impl StepId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "step_{}", self.0)
    }
}

impl serde::Serialize for StepId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for StepId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = String::deserialize(d)?;
        let v = v
            .strip_prefix("step_")
            .ok_or_else(|| serde::de::Error::custom("invalid step id"))?;

        let v = uuid::Uuid::parse_str(v).map_err(serde::de::Error::custom)?;
        Ok(Self(v))
    }
}
