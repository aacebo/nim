#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub num_channels: usize,
    pub depth_multiplier: f64,
    pub depth_divisible_by: usize,
    pub min_depth: usize,
    pub expand_ratio: f64,
    pub output_stride: usize,
    pub first_layer_is_expansion: bool,
    pub finegrained_output: bool,
    pub tf_padding: bool,
    pub layer_norm_eps: f64,
    pub id2label: std::collections::BTreeMap<usize, String>,
}

impl Config {
    pub fn channels(&self, channels: usize) -> usize {
        let value = (channels as f64 * self.depth_multiplier).round() as usize;
        let minimum = self.min_depth.max(self.depth_divisible_by);
        let mut rounded = ((value + self.depth_divisible_by / 2) / self.depth_divisible_by) * self.depth_divisible_by;

        rounded = rounded.max(minimum);

        if rounded * 10 < value * 9 {
            rounded += self.depth_divisible_by;
        }

        rounded
    }

    pub fn output_channels(&self) -> usize {
        match self.finegrained_output && self.depth_multiplier < 1.0 {
            true => 1280,
            false => self.channels(1280),
        }
    }
}
