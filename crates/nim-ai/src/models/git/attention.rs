use candle_core::Tensor;
use candle_nn::{Linear, VarBuilder, linear};

pub struct Attention {
    q: Linear,
    k: Linear,
    v: Linear,
    out: Linear,
    heads: usize,
    head: usize,
}

impl Attention {
    pub fn from_parts(hidden: usize, heads: usize, qkv: VarBuilder, output: VarBuilder) -> candle_core::Result<Self> {
        Ok(Self {
            q: linear(hidden, hidden, qkv.pp("query"))?,
            k: linear(hidden, hidden, qkv.pp("key"))?,
            v: linear(hidden, hidden, qkv.pp("value"))?,
            out: linear(hidden, hidden, output)?,
            heads,
            head: hidden / heads,
        })
    }

    pub fn new(hidden: usize, heads: usize, vars: VarBuilder, q: &str, k: &str, v: &str, out: &str) -> candle_core::Result<Self> {
        Ok(Self {
            q: linear(hidden, hidden, vars.pp(q))?,
            k: linear(hidden, hidden, vars.pp(k))?,
            v: linear(hidden, hidden, vars.pp(v))?,
            out: linear(hidden, hidden, vars.pp(out))?,
            heads,
            head: hidden / heads,
        })
    }

    pub fn forward(&self, xs: &Tensor, mask: Option<&Tensor>) -> candle_core::Result<Tensor> {
        let (batch, seq, _) = xs.dims3()?;
        let shape = |x: Tensor| x.reshape((batch, seq, self.heads, self.head))?.transpose(1, 2)?.contiguous();
        let q = shape(xs.apply(&self.q)?)?;
        let k = shape(xs.apply(&self.k)?)?;
        let v = shape(xs.apply(&self.v)?)?;
        let scores = (q.matmul(&k.transpose(2, 3)?)? / (self.head as f64).sqrt())?;
        let scores = match mask {
            Some(mask) => scores.broadcast_add(mask)?,
            None => scores,
        };
        candle_nn::ops::softmax_last_dim(&scores)?
            .matmul(&v)?
            .transpose(1, 2)?
            .contiguous()?
            .reshape((batch, seq, self.heads * self.head))?
            .apply(&self.out)
    }
}
