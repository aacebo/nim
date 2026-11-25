use candle_core::{DType, Device, Shape, Tensor};
use candle_nn::{Init, VarBuilder, var_builder::SimpleBackend};

pub struct ParlerWeights {
    vars: VarBuilder<'static>,
    vocabulary: usize,
}

impl ParlerWeights {
    pub fn new(vars: VarBuilder<'static>, vocabulary: usize) -> Self {
        Self { vars, vocabulary }
    }

    fn tensor(&self, name: &str, dtype: DType) -> candle_core::Result<Tensor> {
        if self.vars.contains_tensor(name) {
            return self.vars.get_unchecked_dtype(name, dtype);
        }

        if let Some(index) = name
            .strip_prefix("decoder.lm_heads.")
            .and_then(|name| name.strip_suffix(".weight"))
            .and_then(|index| index.parse::<usize>().ok())
            && self.vars.contains_tensor("decoder.lm_heads.weight")
        {
            return self
                .vars
                .get_unchecked_dtype("decoder.lm_heads.weight", dtype)?
                .narrow(0, index * self.vocabulary, self.vocabulary)?
                .contiguous();
        }

        let name = audio_name(name).unwrap_or_else(|| name.to_string());

        if let Some(name) = name.strip_suffix(".weight_g") {
            return self
                .vars
                .get_unchecked_dtype(&format!("{name}.weight"), dtype)?
                .sqr()?
                .sum_keepdim((1, 2))?
                .sqrt();
        }

        if let Some(name) = name.strip_suffix(".weight_v") {
            return self.vars.get_unchecked_dtype(&format!("{name}.weight"), dtype);
        }

        self.vars.get_unchecked_dtype(&name, dtype)
    }
}

impl SimpleBackend for ParlerWeights {
    fn get(&self, shape: Shape, name: &str, _: Init, dtype: DType, _: &Device) -> candle_core::Result<Tensor> {
        let tensor = self.tensor(name, dtype)?;

        if tensor.shape() != &shape {
            return Err(candle_core::Error::Msg(format!(
                "Parler tensor {name} has shape {:?}, expected {shape:?}",
                tensor.shape()
            )));
        }

        Ok(tensor)
    }

    fn get_unchecked(&self, name: &str, dtype: DType, _: &Device) -> candle_core::Result<Tensor> {
        self.tensor(name, dtype)
    }

    fn contains_tensor(&self, name: &str) -> bool {
        self.vars.contains_tensor(name)
    }
}

fn audio_name(name: &str) -> Option<String> {
    let name = name.strip_prefix("audio_encoder.model.")?;

    if name.starts_with("quantizer.") {
        return Some(format!("audio_encoder.{name}"));
    }

    let (side, rest) = name.split_once('.')?;
    let rest = match side {
        "encoder" => rest.strip_prefix("block.")?,
        "decoder" => rest.strip_prefix("model.")?,
        _ => return None,
    };
    let (index, rest) = rest.split_once('.')?;
    let index = index.parse::<usize>().ok()?;
    let mapped = match index {
        0 => format!("conv1.{rest}"),
        5 => format!("snake1.{rest}"),
        6 => format!("conv2.{rest}"),
        1..=4 => {
            let rest = rest.strip_prefix("block.")?;
            let (part, rest) = rest.split_once('.')?;
            let part = part.parse::<usize>().ok()?;
            let part = match (side, part) {
                ("encoder", 0..=2) => format!("res_unit{}.{}", part + 1, residual_name(rest)?),
                ("encoder", 3) => format!("snake1.{rest}"),
                ("encoder", 4) => format!("conv1.{rest}"),
                ("decoder", 0) => format!("snake1.{rest}"),
                ("decoder", 1) => format!("conv_t1.{rest}"),
                ("decoder", 2..=4) => format!("res_unit{}.{}", part - 1, residual_name(rest)?),
                _ => return None,
            };
            format!("block.{}.{part}", index - 1)
        }
        _ => return None,
    };

    Some(format!("audio_encoder.{side}.{mapped}"))
}

fn residual_name(name: &str) -> Option<String> {
    let name = name.strip_prefix("block.")?;
    let (index, rest) = name.split_once('.')?;
    let part = match index {
        "0" => "snake1",
        "1" => "conv1",
        "2" => "snake2",
        "3" => "conv2",
        _ => return None,
    };

    Some(format!("{part}.{rest}"))
}
