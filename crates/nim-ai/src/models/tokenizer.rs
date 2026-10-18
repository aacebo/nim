use std::collections::HashMap;
use std::path::Path;

use tokenizers::models::bpe::BPE;
use tokenizers::models::wordpiece::WordPiece;
use tokenizers::normalizers::BertNormalizer;
use tokenizers::pre_tokenizers::bert::BertPreTokenizer;
use tokenizers::pre_tokenizers::byte_level::ByteLevel;
use tokenizers::processors::bert::BertProcessing;
use tokenizers::{AddedToken, Model, Tokenizer};

use crate::Error;

pub fn from_file(path: impl AsRef<Path>) -> Result<Tokenizer, Error> {
    Tokenizer::from_file(path.as_ref()).map_err(|err| Error::custom("unknown", err))
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

pub fn from_bpe(vocab: impl AsRef<Path>, merges: impl AsRef<Path>, added: impl AsRef<Path>) -> Result<Tokenizer, Error> {
    let bpe = BPE::from_file(&vocab.as_ref().to_string_lossy(), &merges.as_ref().to_string_lossy())
        .build()
        .map_err(|err| Error::custom("unknown", err))?;
    let mut tokenizer = Tokenizer::new(bpe);
    let mut added: Vec<(String, u32)> = serde_json::from_slice::<HashMap<String, u32>>(&std::fs::read(added)?)?
        .into_iter()
        .collect();

    added.sort_by_key(|(_, id)| *id);
    tokenizer.add_special_tokens(&[AddedToken::from("<|endoftext|>", true)]);
    tokenizer.add_special_tokens(
        &added
            .into_iter()
            .map(|(token, _)| AddedToken::from(token, true))
            .collect::<Vec<_>>(),
    );
    tokenizer.with_pre_tokenizer(Some(ByteLevel::default()));
    tokenizer.with_decoder(Some(ByteLevel::default()));

    Ok(tokenizer)
}
