use crate::{
    Context, Split, SplitOptions,
    types::{TextOutput, Word},
};

use super::{AnyModel, Loader, distilbert};

pub enum TextSplitter {
    DistilBertSplitter(distilbert::DistilBertSplitter),
}

impl TextSplitter {
    pub async fn load(repo: &Loader) -> nim_error::Result<Self> {
        let config: distilbert::DistilBertConfig = repo.config().await?;
        Ok(Self::DistilBertSplitter(distilbert::DistilBertSplitter::new(
            repo.vars().await?,
            &config,
        )?))
    }
}

impl Split for TextSplitter {
    fn split<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a str,
        options: SplitOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>> {
        match self {
            Self::DistilBertSplitter(model) => model.split(ctx, input, options),
        }
    }

    fn split_delimited<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a str,
        options: SplitOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Word>>> {
        match self {
            Self::DistilBertSplitter(model) => model.split_delimited(ctx, input, options),
        }
    }
}

impl From<TextSplitter> for AnyModel {
    fn from(model: TextSplitter) -> Self {
        Self::TextSplitter(model)
    }
}
