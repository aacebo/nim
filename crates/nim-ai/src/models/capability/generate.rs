use crate::models::Context;

pub trait Generate: Send + Sync {
    fn generate(&self, ctx: &Context, input: &[&[u8]], options: GenerateOptions<'_>) -> nim_error::Result<Vec<Generation>>;
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Generation {
    pub text: String,
    pub confidence: Option<f64>,
}

#[derive(Debug, Default, Clone)]
pub struct GenerateOptions<'a> {
    pub prompt: Option<&'a str>,
    pub language: Option<&'a str>,
    pub max_len: Option<usize>,
    pub beams: Option<usize>,
}

impl GenerateOptions<'_> {
    pub fn validate(&self) -> nim_error::Result<()> {
        if self.beams == Some(0) || self.max_len == Some(0) {
            return Err(nim_error::Diagnostic::message("beams and max_len must be greater than zero"));
        }

        Ok(())
    }
}
