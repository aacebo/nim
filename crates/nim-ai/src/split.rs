use crate::{
    Context,
    types::{TextOutput, Word},
};

pub trait Split: Send + Sync {
    fn split<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a str,
        options: SplitOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<TextOutput>>>;

    fn split_delimited<'a>(
        &'a self,
        ctx: &'a Context<'_>,
        input: &'a str,
        options: SplitOptions,
    ) -> futures_util::future::BoxFuture<'a, nim_error::Result<Vec<Word>>>;
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct SplitOptions {
    /// max number of generated splits
    pub max_length: Option<usize>,
}
