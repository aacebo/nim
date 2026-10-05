#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Version(u64);

impl Version {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn inc(&mut self) {
        self.0 += 1;
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
