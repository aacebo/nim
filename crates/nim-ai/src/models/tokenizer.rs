use std::collections::HashMap;
use std::path::Path;

use tokenizers::models::bpe::BPE;
use tokenizers::models::wordpiece::WordPiece;
use tokenizers::normalizers::BertNormalizer;
use tokenizers::pre_tokenizers::bert::BertPreTokenizer;
use tokenizers::pre_tokenizers::byte_level::ByteLevel;
use tokenizers::processors::bert::BertProcessing;
use tokenizers::{AddedToken, Model, Tokenizer};

pub fn from_file(path: impl AsRef<Path>) -> nim_error::Result<Tokenizer> {
    Tokenizer::from_file(path.as_ref()).map_err(nim_error::Diagnostic::message)
}

/// The SST-2 and CoNLL-03 checkpoints ship a WordPiece `vocab.txt` and no `tokenizer.json`.
pub fn from_vocab(path: impl AsRef<Path>, lowercase: bool) -> nim_error::Result<Tokenizer> {
    let wordpiece = WordPiece::from_file(&path.as_ref().to_string_lossy())
        .build()
        .map_err(nim_error::Diagnostic::message)?;
    let cls = wordpiece
        .token_to_id("[CLS]")
        .ok_or_else(|| nim_error::Diagnostic::message("vocab is missing [CLS]"))?;
    let sep = wordpiece
        .token_to_id("[SEP]")
        .ok_or_else(|| nim_error::Diagnostic::message("vocab is missing [SEP]"))?;

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

pub fn from_bpe(vocab: impl AsRef<Path>, merges: impl AsRef<Path>, added: impl AsRef<Path>) -> nim_error::Result<Tokenizer> {
    let bpe = BPE::from_file(&vocab.as_ref().to_string_lossy(), &merges.as_ref().to_string_lossy())
        .build()
        .map_err(nim_error::Diagnostic::message)?;
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
