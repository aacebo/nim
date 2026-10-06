use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::distilbert;

use super::config::Config;
use crate::{Error, models::Forward};

pub struct DistilBert {
    inner: distilbert::DistilBertModel,
}

impl DistilBert {
    pub fn new(vars: VarBuilder, config: &Config) -> Result<Self, Error> {
        Ok(Self {
            inner: distilbert::DistilBertModel::load(vars, &config.to_candle()?)?,
        })
    }

    pub fn forward(&self, ids: &Tensor, padding: &Tensor) -> Result<Tensor, Error> {
        Ok(self.inner.forward(ids, padding)?)
    }
}

impl Forward for DistilBert {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, padding): Self::Input) -> Result<Self::Output, Error> {
        self.forward(&ids, &padding)
    }
}
