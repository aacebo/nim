mod config;
mod decoder;
mod model;
mod processor;
mod synthesizer;
mod voice;
mod weights;

pub use config::ParlerConfig;
pub use processor::ParlerProcessor;
pub use synthesizer::ParlerSynthesizer;
pub use voice::*;

pub const MODEL: &str = "parler-tts/parler-tts-mini-v1.1";
