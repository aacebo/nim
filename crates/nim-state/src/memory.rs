use crate::{AnnotationId, ContextId, Embedding, Version};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct MemoryId(uuid::Uuid);

impl std::fmt::Display for MemoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "memory_{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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
pub struct Memory {
    pub id: MemoryId,

    /// version gets incremented once
    /// per update.
    pub version: Version,

    #[serde(rename = "type")]
    pub ty: MemoryType,

    /// the contexts this memory was created from.
    pub contexts: Vec<ContextId>,

    /// the generated annotations describing the dimensions
    /// of the memory that was inferred.
    pub annotations: Vec<AnnotationId>,

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

    /// vector embedding used for search.
    pub embedding: Option<Embedding>,

    /// how many times has this memory been recalled since creation.
    pub recalls: u64,

    /// the most recent recall timestamp.
    pub recalled_at: Option<chrono::DateTime<chrono::Utc>>,

    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
