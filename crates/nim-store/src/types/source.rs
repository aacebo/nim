use super::Region;

#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Source {
    System,
    Region(Region),
    Memory(uuid::Uuid),
    Entity(uuid::Uuid),
}
