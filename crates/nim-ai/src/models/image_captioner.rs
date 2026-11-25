use crate::{Caption, CaptionOptions, Context, types::TextOutput};

use super::{AnyModel, Loader, git};

pub enum ImageCaptioner {
    GitCaptioner(git::GitCaptioner),
}

impl ImageCaptioner {
    pub async fn load(repo: &Loader) -> nim_error::Result<Self> {
        let config: git::GitConfig = repo.config().await?;
        let processor: git::GitProcessor = repo.json("preprocessor_config.json").await?;
        Ok(Self::GitCaptioner(git::GitCaptioner::new(
            config,
            processor,
            repo.vars().await?,
        )?))
    }
}

impl Caption for ImageCaptioner {
    fn caption<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a [&'a [u8]],
        options: CaptionOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        match self {
            Self::GitCaptioner(model) => model.caption(ctx, input, options),
        }
    }
}

impl From<ImageCaptioner> for AnyModel {
    fn from(model: ImageCaptioner) -> Self {
        Self::ImageCaptioner(model)
    }
}
