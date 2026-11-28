mod generator;

pub use generator::Qwen2Generator;

pub const MODEL: &str = "Qwen/Qwen2.5-1.5B-Instruct";
pub const QUANTIZED_MODEL: &str = "Qwen/Qwen2.5-1.5B-Instruct-GGUF";
pub const WEIGHTS: &str = "qwen2.5-1.5b-instruct-q4_k_m.gguf";
