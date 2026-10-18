use super::Region;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Source {
    System,
    Region(Region),
    Memory(uuid::Uuid),
    Entity(uuid::Uuid),
    Prediction(uuid::Uuid),
    Fact(uuid::Uuid),
}
