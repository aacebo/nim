use std::collections::HashMap;
use std::path::Path;

use tokenizers::models::bpe::BPE;
use tokenizers::models::wordpiece::WordPiece;
use tokenizers::normalizers::BertNormalizer;
use tokenizers::pre_tokenizers::bert::BertPreTokenizer;
use tokenizers::pre_tokenizers::byte_level::ByteLevel;
use tokenizers::processors::bert::BertProcessing;
use tokenizers::processors::template::TemplateProcessing;
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

pub fn from_bpe(
    vocab: impl AsRef<Path>,
    merges: impl AsRef<Path>,
    added: Option<&Path>,
    special: Option<&Path>,
) -> nim_error::Result<Tokenizer> {
    let bpe = BPE::from_file(&vocab.as_ref().to_string_lossy(), &merges.as_ref().to_string_lossy())
        .build()
        .map_err(nim_error::Diagnostic::message)?;
    let mut tokenizer = Tokenizer::new(bpe);

    if let Some(added) = added {
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
    } else if let Some(special) = special {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum SpecialToken {
            Name(String),
            Config { content: String },
        }

        impl SpecialToken {
            fn content(self) -> String {
                match self {
                    Self::Name(value) | Self::Config { content: value } => value,
                }
            }
        }

        let special = serde_json::from_slice::<HashMap<String, SpecialToken>>(&std::fs::read(special)?)?;
        let bos = special.get("bos_token").map(|token| match token {
            SpecialToken::Name(value) | SpecialToken::Config { content: value } => value.clone(),
        });
        let eos = special.get("eos_token").map(|token| match token {
            SpecialToken::Name(value) | SpecialToken::Config { content: value } => value.clone(),
        });

        tokenizer.add_special_tokens(
            &special
                .into_values()
                .map(SpecialToken::content)
                .map(|token| AddedToken::from(token, true))
                .collect::<Vec<_>>(),
        );

        if let (Some(bos), Some(eos)) = (bos, eos) {
            let bos_id = tokenizer
                .token_to_id(&bos)
                .ok_or_else(|| nim_error::Diagnostic::message("BPE vocabulary is missing the BOS token"))?;
            let eos_id = tokenizer
                .token_to_id(&eos)
                .ok_or_else(|| nim_error::Diagnostic::message("BPE vocabulary is missing the EOS token"))?;
            let processor = TemplateProcessing::builder()
                .try_single(format!("{bos} $A {eos}"))
                .map_err(nim_error::Diagnostic::message)?
                .try_pair(format!("{bos} $A {eos} {eos} $B:1 {eos}:1"))
                .map_err(nim_error::Diagnostic::message)?
                .special_tokens(vec![(bos, bos_id), (eos, eos_id)])
                .build()
                .map_err(nim_error::Diagnostic::message)?;

            tokenizer.with_post_processor(Some(processor));
        }
    }

    tokenizer.with_pre_tokenizer(Some(ByteLevel::default()));
    tokenizer.with_decoder(Some(ByteLevel::default()));

    Ok(tokenizer)
}
