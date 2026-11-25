use crate::{Artifact, Context, Embedding, Source, Version};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Memory {
    pub id: MemoryId,

    /// version gets incremented once per update.
    pub version: Version,

    #[serde(rename = "type")]
    pub ty: MemoryType,

    /// who/what/where the memory was derived from.
    pub sources: Vec<Source>,

    /// any files/text associated with the memory.
    pub artifacts: Vec<Artifact>,

    /// semantic/episodic links
    pub relations: Vec<MemoryId>,

    /// the contexts this memory was created from.
    pub contexts: Vec<Context>,

    /// summary of memory content
    pub summary: Option<String>,

    /// embedding of summary
    #[serde(skip)]
    pub embedding: Option<Embedding>,

    /// ### Filter
    ///
    /// the subjective or physical distinctiveness of an event, object,
    /// or piece of information that makes it stand out, grab attention,
    /// and encode more deeply into memory.
    pub salience: f32,

    /// ### Trace
    ///
    /// the physical durability of the memory trace (the engram) in the brain.
    /// Built primarily in the hippocampus and consolidated into the cortex,
    /// strength determines how resistant a memory is to forgetting over time.
    pub strength: f32,

    /// ### Judgement
    ///
    /// metacognitive belief in the accuracy of a retrieved memory.
    /// It is your subjective feeling of
    /// "I am 100% sure that happened" versus "I think that happened, but I might be wrong."
    pub confidence: f32,

    /// how many times has this memory been recalled since creation.
    pub recalls: u64,

    /// the most recent recall timestamp.
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<nim_store::MemoryRow> for Memory {
    type Error = nim_error::Diagnostic;

    fn try_from(value: nim_store::MemoryRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            version: value.version.into(),
            ty: serde_json::from_value(serde_json::Value::String(value.ty))?,
            sources: serde_json::from_value(value.sources)?,
            artifacts: Default::default(),
            relations: Default::default(),
            contexts: serde_json::from_value(value.contexts)?,
            summary: value.summary,
            embedding: value.embedding.map(Into::into),
            salience: value.salience,
            strength: value.strength,
            confidence: value.confidence,
            recalls: value.recalls as u64,
            recalled_at: value.recalled_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryId(uuid::Uuid);

impl MemoryId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl From<uuid::Uuid> for MemoryId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<MemoryId> for uuid::Uuid {
    fn from(value: MemoryId) -> Self {
        value.0
    }
}

impl std::fmt::Display for MemoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mem_{}", self.0.simple())
    }
}

impl std::str::FromStr for MemoryId {
    type Err = nim_error::Diagnostic;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value
            .strip_prefix("mem_")
            .ok_or_else(|| nim_error::Diagnostic::message("invalid memory id"))?;

        Ok(Self(uuid::Uuid::parse_str(value)?))
    }
}

impl serde::Serialize for MemoryId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for MemoryId {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    /// ### Narrative
    ///
    /// personal, time-stamped life experiences.
    ///
    /// #### Example
    ///
    /// the time you got flat tire on the highway last summer.
    Episodic,

    /// ### Manual
    ///
    /// factual knowledge, concepts, and rules of the world
    /// without a specific time attachment.
    ///
    /// #### Example
    ///
    /// You know that a red octagonal sign means "Stop."
    Semantic,

    /// ### Blueprint
    ///
    /// motor skills and muscle memory.
    ///
    /// #### Example
    ///
    /// You automatically steer, use the pedals, and shift gears
    /// without consciously thinking about your feet or hands.
    Procedural,

    /// ### Dashboard
    ///
    /// immediate, active data manipulation.
    ///
    /// #### Example
    ///
    /// You mentally track the silver SUV in your blind spot while
    /// calculating if you have enough time to make a left turn.
    Working,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Episodic => write!(f, "episodic"),
            Self::Semantic => write!(f, "semantic"),
            Self::Procedural => write!(f, "procedural"),
            Self::Working => write!(f, "working"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryRelation {
    pub source_id: MemoryId,
    pub target_id: MemoryId,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<nim_store::MemoryRelationRow> for MemoryRelation {
    fn from(value: nim_store::MemoryRelationRow) -> Self {
        Self {
            source_id: value.source_id.into(),
            target_id: value.target_id.into(),
            created_at: value.created_at,
        }
    }
}
