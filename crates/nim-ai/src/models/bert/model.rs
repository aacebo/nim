use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::bert;

use crate::models::Forward;

use super::config::Config;

pub struct Bert {
    inner: bert::BertModel,
}

impl Bert {
    pub fn new(vars: VarBuilder, config: &Config) -> nim_error::Result<Self> {
        Ok(Self {
            inner: bert::BertModel::load(vars, &bert::Config::from(config))?,
        })
    }

    /// `mask` is the keep-mask (1 = real token); `BertModel` widens it internally.
    pub fn forward(&self, ids: &Tensor, mask: &Tensor) -> nim_error::Result<Tensor> {
        let types = ids.zeros_like()?;
        Ok(self.inner.forward(ids, &types, Some(mask))?)
    }
}

impl Forward for Bert {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, mask): Self::Input) -> nim_error::Result<Self::Output> {
        self.forward(&ids, &mask)
    }
}
