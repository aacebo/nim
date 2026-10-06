use std::path::Path;

use tokenizers::models::wordpiece::WordPiece;
use tokenizers::normalizers::BertNormalizer;
use tokenizers::pre_tokenizers::bert::BertPreTokenizer;
use tokenizers::processors::bert::BertProcessing;
use tokenizers::{Model, Tokenizer};

use crate::Error;

pub fn from_file(path: impl AsRef<Path>) -> Result<Tokenizer, Error> {
    Ok(Tokenizer::from_file(path.as_ref()).map_err(|err| Error::custom("unknown", err))?)
}

/// The SST-2 and CoNLL-03 checkpoints ship a WordPiece `vocab.txt` and no `tokenizer.json`.
pub fn from_vocab(path: impl AsRef<Path>, lowercase: bool) -> Result<Tokenizer, Error> {
    let wordpiece = WordPiece::from_file(&path.as_ref().to_string_lossy())
        .build()
        .map_err(|err| Error::custom("unknown", err))?;
    let cls = wordpiece
        .token_to_id("[CLS]")
        .ok_or_else(|| Error::custom("ai::invalid", "vocab is missing [CLS]"))?;
    let sep = wordpiece
        .token_to_id("[SEP]")
        .ok_or_else(|| Error::custom("ai::invalid", "vocab is missing [SEP]"))?;

    let mut tokenizer = Tokenizer::new(wordpiece);

    tokenizer
        .with_normalizer(Some(BertNormalizer::new(true, true, None, lowercase)))
        .with_pre_tokenizer(Some(BertPreTokenizer))
        .with_post_processor(Some(BertProcessing::new(
            ("[SEP]".to_string(), sep),
            ("[CLS]".to_string(), cls),
        )));

    Ok(tokenizer)
}
