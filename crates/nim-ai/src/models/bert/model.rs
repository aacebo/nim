use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::bert;

use super::config::Config;
use crate::{Error, models::Forward};

pub struct Bert {
    inner: bert::BertModel,
}

impl Bert {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self, Error> {
        Ok(Self {
            inner: bert::BertModel::load(vars, &bert::Config::from(config))?,
        })
    }

    /// `mask` is the keep-mask (1 = real token); `BertModel` widens it internally.
    pub fn forward(&self, ids: &Tensor, mask: &Tensor) -> Result<Tensor, Error> {
        let types = ids.zeros_like()?;
        Ok(self.inner.forward(ids, &types, Some(mask))?)
    }
}

impl Forward for Bert {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, mask): Self::Input) -> Result<Self::Output, Error> {
        self.forward(&ids, &mask)
    }
}
