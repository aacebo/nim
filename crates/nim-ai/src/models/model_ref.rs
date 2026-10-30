use std::str::FromStr;
use std::sync::Arc;

use candle_core::{DType, Device};

use crate::Error;
use crate::models::{Loader, ModelId, Provider};
use crate::resources::{DataSource, Repository, Resource, Uri};

use crate::clients::fs::FileSystemRepository;
use crate::clients::hf::HuggingFace;
use crate::clients::http::HttpRepository;
use crate::clients::openai::OpenAI;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModelRef {
    Local(LocalModel),
    Remote(RemoteModel),
}

impl ModelRef {
    pub fn hub(id: ModelId) -> Self {
        Self::Local(LocalModel::Hub(id))
    }

    pub fn local(uri: Uri) -> Self {
        Self::Local(LocalModel::Path(Resource::new(uri)))
    }

    pub fn remote(provider: Provider, id: ModelId) -> Self {
        Self::Remote(RemoteModel::new(provider, id))
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        if let Self::Remote(remote) = self {
            self = Self::Remote(remote.base_url(url));
        }

        self
    }

    pub fn name(&self) -> String {
        match self {
            Self::Local(v) => v.name(),
            Self::Remote(v) => v.id.to_string(),
        }
    }

    /// The weights-bearing model, or an error naming the remote one that has none.
    pub fn local_or_err(&self) -> Result<&LocalModel, Error> {
        match self {
            Self::Local(model) => Ok(model),
            Self::Remote(remote) => Err(Error::custom(
                "ai::invalid",
                format!("{remote} is a remote model and has no weights"),
            )),
        }
    }
}

impl FromStr for ModelRef {
    type Err = Error;

    fn from_str(model: &str) -> std::result::Result<Self, Self::Err> {
        let scheme = model.starts_with("file://") || model.starts_with("http://") || model.starts_with("https://");

        if scheme || std::path::Path::new(model).is_dir() {
            Ok(ModelRef::local(model.parse::<Uri>()?))
        } else {
            Ok(ModelRef::hub(model.parse::<ModelId>()?))
        }
    }
}

impl std::fmt::Display for ModelRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(model) => write!(f, "{model}"),
            Self::Remote(remote) => write!(f, "{remote}"),
        }
    }
}

/// A model whose weights we load and run ourselves. Every variant has weights, so `repository`
/// and `loader` are total -- a model with no weights cannot be built as a `LocalModel`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalModel {
    Hub(ModelId),
    Path(Resource),
}

impl LocalModel {
    pub fn name(&self) -> String {
        match self {
            Self::Hub(v) => v.to_string(),
            Self::Path(v) => v.name().to_string(),
        }
    }

    pub fn loader(&self, device: Device, dtype: DType) -> Result<Loader, Error> {
        Ok(Loader::new(self.repository()?, device, dtype))
    }

    pub fn repository(&self) -> Result<Arc<dyn Repository>, Error> {
        match self {
            Self::Hub(id) => Ok(Arc::new(HuggingFace.load(id)?)),
            Self::Path(resource) => match &resource.uri {
                Uri::Local(path) => Ok(Arc::new(FileSystemRepository::new(path))),
                Uri::Http(_) => Ok(Arc::new(HttpRepository::new(resource.uri.clone()))),
            },
        }
    }
}

impl std::fmt::Display for LocalModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hub(id) => write!(f, "{id}"),
            Self::Path(resource) => write!(f, "{resource}"),
        }
    }
}

/// A model we call over the wire. It has an endpoint, never weights.
#[derive(Debug, Clone)]
pub struct RemoteModel {
    pub provider: Provider,
    pub id: ModelId,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

impl RemoteModel {
    pub fn new(provider: Provider, id: ModelId) -> Self {
        Self {
            provider,
            id,
            base_url: None,
            api_key: None,
        }
    }

    pub fn base_url(mut self, url: Option<String>) -> Self {
        self.base_url = url;
        self
    }

    pub fn api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key;
        self
    }

    pub fn client(&self) -> OpenAI {
        match self.provider {
            Provider::OpenAI => OpenAI::new(self.id.clone(), self.base_url.clone(), self.api_key.clone()),
        }
    }
}

impl PartialEq for RemoteModel {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider && self.id == other.id && self.base_url == other.base_url
    }
}

impl Eq for RemoteModel {}

impl std::hash::Hash for RemoteModel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.provider.hash(state);
        self.id.hash(state);
        self.base_url.hash(state);
    }
}

impl std::fmt::Display for RemoteModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.provider, self.id)
    }
}
