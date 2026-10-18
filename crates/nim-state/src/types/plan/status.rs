#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Active,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Submitted => write!(f, "submitted"),
            Self::Approved => write!(f, "approved"),
            Self::Rejected => write!(f, "rejected"),
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<nim_store::PlanStatus> for PlanStatus {
    fn from(value: nim_store::PlanStatus) -> Self {
        match value {
            nim_store::PlanStatus::Draft => Self::Draft,
            nim_store::PlanStatus::Submitted => Self::Submitted,
            nim_store::PlanStatus::Approved => Self::Approved,
            nim_store::PlanStatus::Rejected => Self::Rejected,
            nim_store::PlanStatus::Active => Self::Active,
            nim_store::PlanStatus::Completed => Self::Completed,
            nim_store::PlanStatus::Failed => Self::Failed,
            nim_store::PlanStatus::Cancelled => Self::Cancelled,
            _ => unreachable!(),
        }
    }
}

impl From<PlanStatus> for nim_store::PlanStatus {
    fn from(value: PlanStatus) -> Self {
        match value {
            PlanStatus::Draft => Self::Draft,
            PlanStatus::Submitted => Self::Submitted,
            PlanStatus::Approved => Self::Approved,
            PlanStatus::Rejected => Self::Rejected,
            PlanStatus::Active => Self::Active,
            PlanStatus::Completed => Self::Completed,
            PlanStatus::Failed => Self::Failed,
            PlanStatus::Cancelled => Self::Cancelled,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    NotStarted,
    Active,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotStarted => write!(f, "not_started"),
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}
