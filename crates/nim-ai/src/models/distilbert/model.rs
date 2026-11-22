use candle_core::Tensor;
use candle_nn::VarBuilder;
use candle_transformers::models::distilbert;

use crate::models::Forward;

use super::config::DistilBertConfig;

pub struct DistilBert {
    inner: distilbert::DistilBertModel,
}

impl DistilBert {
    pub fn new(vars: VarBuilder, config: &DistilBertConfig) -> nim_error::Result<Self> {
        Ok(Self {
            inner: distilbert::DistilBertModel::load(vars, &config.to_candle()?)?,
        })
    }

    pub fn forward(&self, ids: &Tensor, padding: &Tensor) -> nim_error::Result<Tensor> {
        let padding = if padding.rank() == 2 {
            padding.unsqueeze(1)?.unsqueeze(1)?
        } else {
            padding.clone()
        };
        Ok(self.inner.forward(ids, &padding)?)
    }
}

impl Forward for DistilBert {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, padding): Self::Input) -> nim_error::Result<Self::Output> {
        self.forward(&ids, &padding)
    }
}
