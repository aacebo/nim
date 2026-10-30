mod captioner;
mod config;
mod model;
mod processor;

pub use captioner::*;
pub use config::*;

pub fn tokenizer() -> Result<tokenizers::Tokenizer, crate::Error> {
    use std::path::Path;

    use crate::clients::hf::HuggingFace;
    use crate::models::ModelId;
    use crate::models::tokenizer;
    use crate::resources::{DataSource, Repository};

    let id = "wsntxxn/audiocaps-simple-tokenizer".parse::<ModelId>()?;
    let repo = HuggingFace.load(&id)?;
    tokenizer::from_file(repo.resolve(Path::new("tokenizer.json"))?.local_path()?)
}
