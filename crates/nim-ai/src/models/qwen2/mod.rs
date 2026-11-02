mod generator;

pub use candle_transformers::models::qwen2::Config;
pub use generator::*;

pub const MODEL: &str = "Qwen/Qwen2.5-3B-Instruct";
pub const QUANTIZED_MODEL: &str = "Qwen/Qwen2.5-3B-Instruct-GGUF";
pub const WEIGHTS: &str = "qwen2.5-3b-instruct-q4_k_m.gguf";
