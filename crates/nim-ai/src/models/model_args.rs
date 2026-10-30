use crate::{
    Error,
    models::{ModelId, ModelRef, Provider},
};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelArgs {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

impl ModelArgs {
    pub fn resolve(self, default: ModelRef) -> Result<ModelRef, Error> {
        let Some(provider) = self.provider else {
            let Some(model) = self.model else {
                let mut model = default;
                if let ModelRef::Remote(remote) = &mut model {
                    if self.api_key.is_some() {
                        remote.api_key = self.api_key;
                    }
                }

                return Ok(model);
            };

            return model.parse();
        };

        let provider: Provider = provider.parse()?;
        let id: ModelId = self
            .model
            .ok_or(Error::custom("ai::config", "model is required when provider is set"))?
            .parse()?;

        let mut model = ModelRef::remote(provider, id).base_url(self.base_url);
        if let ModelRef::Remote(remote) = &mut model {
            remote.api_key = self.api_key;
        }

        Ok(model)
    }
}
