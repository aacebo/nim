#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
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

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum StepStatus {
    NotStarted,
    Active,
    Completed,
    Failed,
    Cancelled,
}
