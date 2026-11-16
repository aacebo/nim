use candle_core::{DType, Tensor};
use candle_nn::VarBuilder;

use crate::models::{Context, Embed, Forward};

use super::config::Config;
use super::model::Bert;

pub struct Embedder {
    bert: Bert,
}

impl Embedder {
    pub fn new(vars: VarBuilder, config: &Config) -> nim_error::Result<Self> {
        Ok(Self {
            bert: Bert::new(vars, config)?,
        })
    }

    /// Sentence vectors: mean-pool the token states over the mask, then L2 normalize.
    pub fn forward(&self, ids: &Tensor, mask: &Tensor) -> nim_error::Result<Tensor> {
        let hidden = self.bert.forward(ids, mask)?;
        normalize(&pool(&hidden, mask)?)
    }
}

fn pool(hidden: &Tensor, mask: &Tensor) -> nim_error::Result<Tensor> {
    let mask = mask.to_dtype(DType::F32).and_then(|mask| mask.unsqueeze(2))?;

    let summed = hidden.broadcast_mul(&mask).and_then(|v| v.sum(1))?;
    let counts = mask.sum(1)?;
    Ok(summed.broadcast_div(&counts)?)
}

fn normalize(v: &Tensor) -> nim_error::Result<Tensor> {
    let norm = v.sqr().and_then(|v| v.sum_keepdim(1)).and_then(|v| v.sqrt())?;

    Ok(v.broadcast_div(&norm)?)
}

impl Forward for Embedder {
    type Input = (Tensor, Tensor);
    type Output = Tensor;

    fn forward(&self, (ids, mask): Self::Input) -> nim_error::Result<Self::Output> {
        self.forward(&ids, &mask)
    }
}

impl Embed for Embedder {
    fn embed(&self, ctx: &Context, input: &[&[u8]]) -> nim_error::Result<Vec<Vec<f32>>> {
        let text = input
            .iter()
            .map(|bytes| std::str::from_utf8(bytes).map_err(nim_error::Diagnostic::from))
            .collect::<Result<Vec<_>, _>>()?;
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let batch = ctx.encode(&text)?;
        Ok(self.forward(&batch.ids, &batch.mask)?.to_vec2::<f32>()?)
    }
}
