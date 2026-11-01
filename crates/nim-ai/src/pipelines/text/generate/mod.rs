mod config;
mod constraints;

use candle_core::{Device, Tensor};
use candle_nn::ops;
pub use config::Config;

use crate::{Error, models::bart::Bart};

pub fn run(model: &mut Bart, config: &Config, input: &Tensor, device: &Device) -> Result<(Vec<u32>, Option<f64>), Error> {
    let beams = config.beams;

    model.reset_kv_cache();

    let encoder_xs = model.encode(input)?;
    let (_, seq, hidden) = encoder_xs.dims3()?;
    let encoder_xs = encoder_xs.expand((beams, seq, hidden)).and_then(|xs| xs.contiguous())?;

    let mut sequences: Vec<Vec<u32>> = vec![vec![config.decoder_start_token_id]; beams];
    let mut scores: Vec<f64> = (0..beams).map(|i| if i == 0 { 0.0 } else { f64::NEG_INFINITY }).collect();
    let mut likelihoods = vec![(0.0, 0usize); beams];
    let mut hypotheses: Vec<(f64, Vec<u32>, (f64, usize))> = Vec::new();
    let mut tokens = vec![config.decoder_start_token_id; beams];

    for past_kv_len in 0..config.max_length {
        let input = Tensor::from_vec(tokens.clone(), (beams, 1), device)?;
        let logits = model
            .decode(&input, &encoder_xs, past_kv_len)
            .and_then(|logits| logits.squeeze(1))?;
        let probabilities = ops::log_softmax(&logits, 1).and_then(|probs| probs.to_vec2::<f32>())?;
        let mut log_probs = probabilities.clone();
        let generated = sequences[0].len() - 1;
        let forced = (generated == 0 && config.forced_bos_token_id.is_some())
            || (generated + 1 >= config.max_length && config.forced_eos_token_id.is_some());

        for (beam, row) in log_probs.iter_mut().enumerate() {
            constraints::apply(row, config, &sequences[beam], generated);
        }

        let mut candidates: Vec<(f64, usize, u32)> = Vec::new();

        for (beam, row) in log_probs.iter().enumerate() {
            if scores[beam].is_infinite() && scores[beam] < 0.0 {
                continue;
            }

            for (token, log_prob) in row.iter().enumerate() {
                candidates.push((scores[beam] + *log_prob as f64, beam, token as u32));
            }
        }

        // Only the top 2*beams matter (2x so beams finishing on EOS still leave `beams` live
        // continuations). Partition rather than sort all beams*vocab candidates.
        let wanted = (beams * 2).min(candidates.len());
        candidates.select_nth_unstable_by(wanted - 1, |a, b| b.0.total_cmp(&a.0));
        candidates.truncate(wanted);
        candidates.sort_by(|a, b| b.0.total_cmp(&a.0));

        let mut next: Vec<(f64, usize, u32)> = Vec::with_capacity(beams);

        for (score, beam, token) in candidates {
            if token == config.eos_token_id {
                let mut sequence = sequences[beam].clone();
                sequence.push(token);
                let mut likelihood = likelihoods[beam];

                if !forced {
                    likelihood.0 += probabilities[beam][token as usize] as f64;
                    likelihood.1 += 1;
                }

                hypotheses.push((normalize(score, &sequence, config), sequence, likelihood));
                continue;
            }

            next.push((score, beam, token));

            if next.len() == beams {
                break;
            }
        }

        if next.is_empty() || hypotheses.len() >= beams {
            break;
        }

        // Follow the surviving beams. Skipping this makes a beam read whichever history
        // happens to sit in its slot.
        let order: Vec<u32> = next.iter().map(|(_, beam, _)| *beam as u32).collect();
        let order = Tensor::from_vec(order, next.len(), device)?;
        model.reorder_kv_cache(&order)?;
        let previous = sequences.clone();
        let previous_likelihoods = likelihoods.clone();

        sequences.clear();
        likelihoods.clear();
        scores.clear();
        tokens.clear();

        for (score, beam, token) in &next {
            let mut sequence = previous[*beam].clone();
            sequence.push(*token);

            let mut likelihood = previous_likelihoods[*beam];

            if !forced {
                likelihood.0 += probabilities[*beam][*token as usize] as f64;
                likelihood.1 += 1;
            }

            likelihoods.push(likelihood);
            sequences.push(sequence);
            scores.push(*score);
            tokens.push(*token);
        }
    }

    if hypotheses.is_empty() {
        let best = scores
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .map(|(index, _)| index)
            .unwrap_or(0);

        return Ok((sequences.swap_remove(best), confidence(likelihoods[best])));
    }

    hypotheses.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (_, tokens, likelihood) = hypotheses.swap_remove(0);
    Ok((tokens, confidence(likelihood)))
}

/// HuggingFace's length penalty: `score / len^penalty`. Applied when a hypothesis completes,
/// never to the running beam scores.
fn normalize(score: f64, sequence: &[u32], config: &Config) -> f64 {
    score / (sequence.len() as f64).powf(config.length_penalty)
}

fn confidence((sum, count): (f64, usize)) -> Option<f64> {
    if count == 0 {
        return None;
    }

    let confidence = (sum / count as f64).exp();
    confidence.is_finite().then(|| confidence.clamp(0.0, 1.0))
}
