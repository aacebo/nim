use std::path::{Path, PathBuf};
use std::sync::Arc;

use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use tokenizers::Tokenizer;

use super::{Repository, tokenizer};

pub struct Loader {
    repo: Arc<dyn Repository>,
    device: Device,
    dtype: DType,
}

impl Loader {
    pub fn new(repo: Arc<dyn Repository>, device: Device, dtype: DType) -> Self {
        Self { repo, device, dtype }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub async fn config<T: serde::de::DeserializeOwned>(&self) -> nim_error::Result<T> {
        self.json("config.json").await
    }

    pub async fn tokenizer(&self) -> nim_error::Result<Tokenizer> {
        if let Ok(path) = self.resolve(Path::new("tokenizer.json")).await {
            return tokenizer::from_file(path);
        }

        if self.repo.exists(Path::new("vocab.json")).await {
            let vocab = self.resolve(Path::new("vocab.json")).await?;
            let merges = self.resolve(Path::new("merges.txt")).await?;
            let added = if self.repo.exists(Path::new("added_tokens.json")).await {
                Some(self.resolve(Path::new("added_tokens.json")).await?)
            } else {
                None
            };

            let special = if self.repo.exists(Path::new("special_tokens_map.json")).await {
                Some(self.resolve(Path::new("special_tokens_map.json")).await?)
            } else {
                None
            };

            return tokenizer::from_bpe(vocab, merges, added.as_deref(), special.as_deref());
        }

        let path = self.resolve(Path::new("vocab.txt")).await?;
        tokenizer::from_vocab(path, self.lowercase().await?)
    }

    pub async fn vars(&self) -> nim_error::Result<VarBuilder<'static>> {
        let paths = if self.repo.exists(Path::new("model.safetensors.index.json")).await {
            #[derive(serde::Deserialize)]
            struct Index {
                weight_map: std::collections::BTreeMap<String, String>,
            }

            let index: Index = self.json("model.safetensors.index.json").await?;
            let files = index.weight_map.into_values().collect::<std::collections::BTreeSet<_>>();

            if files.is_empty() {
                return Err(nim_error::Diagnostic::message("safetensors index contains no weights"));
            }

            let mut paths = Vec::with_capacity(files.len());

            for file in files {
                paths.push(self.resolve(Path::new(&file)).await?);
            }

            paths
        } else if self.repo.exists(Path::new("model.safetensors")).await {
            vec![self.resolve(Path::new("model.safetensors")).await?]
        } else {
            Vec::new()
        };

        if !paths.is_empty() {
            return Ok(unsafe { VarBuilder::from_mmaped_safetensors(&paths, self.dtype, &self.device) }?);
        }

        Ok(VarBuilder::from_pth(
            self.resolve(Path::new("pytorch_model.bin")).await?,
            self.dtype,
            &self.device,
        )?)
    }

    pub async fn json<T: serde::de::DeserializeOwned>(&self, file: &str) -> nim_error::Result<T> {
        let path = self.resolve(Path::new(file)).await?;
        let bytes = tokio::fs::read(path).await?;

        Ok(serde_json::from_slice(&bytes)?)
    }

    pub async fn string(&self, file: &str) -> nim_error::Result<String> {
        Ok(tokio::fs::read_to_string(self.resolve(Path::new(file)).await?).await?)
    }

    pub async fn chat_template(&self) -> nim_error::Result<hf_chat_template::ChatTemplate> {
        let config: hf_chat_template::TokenizerConfig = self.json("tokenizer_config.json").await?;

        if self.repo.exists(Path::new("chat_template.jinja")).await {
            return Ok(hf_chat_template::ChatTemplate::from_template_and_config(
                &self.string("chat_template.jinja").await?,
                &config,
            )?);
        }

        Ok(hf_chat_template::ChatTemplate::from_tokenizer_config(&config)?)
    }

    pub(crate) async fn resolve(&self, path: &Path) -> nim_error::Result<PathBuf> {
        Ok(self.repo.resolve(path).await?.local_path()?.to_path_buf())
    }

    /// ModelRef-specific: SST-2 is uncased, CoNLL-03 is cased. Lowercasing a cased model wrecks it.
    async fn lowercase(&self) -> nim_error::Result<bool> {
        #[derive(serde::Deserialize)]
        struct Config {
            #[serde(default)]
            do_lower_case: bool,
        }

        if !self.repo.exists(Path::new("tokenizer_config.json")).await {
            return Ok(false);
        }

        let config: Config = self.json("tokenizer_config.json").await?;
        Ok(config.do_lower_case)
    }
}
