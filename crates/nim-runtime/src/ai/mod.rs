#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "vision")]
mod images;
mod text;

#[cfg(feature = "audio")]
pub use audio::*;
#[cfg(feature = "vision")]
pub use images::*;
pub use text::*;

pub struct Pipelines<'a> {
    #[cfg(feature = "audio")]
    _audio: AudioPipelines<'a>,
    #[cfg(feature = "vision")]
    _images: ImagePipelines<'a>,
    _text: TextPipelines<'a>,
}

impl<'a> Pipelines<'a> {
    pub fn new(pool: &'a nim_store::PgPool, signals: std::sync::Arc<nim_signals::Client>) -> Self {
        Self {
            #[cfg(feature = "audio")]
            _audio: AudioPipelines::new(pool, signals.clone()),
            #[cfg(feature = "vision")]
            _images: ImagePipelines::new(pool, signals.clone()),
            _text: TextPipelines::new(pool, signals),
        }
    }

    #[cfg(feature = "audio")]
    pub fn audio(&self) -> &AudioPipelines<'a> {
        &self._audio
    }

    #[cfg(feature = "vision")]
    pub fn images(&self) -> &ImagePipelines<'a> {
        &self._images
    }

    pub fn text(&self) -> &TextPipelines<'a> {
        &self._text
    }
}
