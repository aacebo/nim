#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Version(u64);

impl Version {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn inc(&mut self) {
        self.0 += 1;
    }
}

impl From<u64> for Version {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i32> for Version {
    fn from(value: i32) -> Self {
        Self(value as u64)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
