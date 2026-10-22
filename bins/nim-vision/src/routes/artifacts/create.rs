use nim::{signals::Signal, state::Artifact};

use crate::{Error, SignalContext};

#[tracing::instrument(skip(ctx))]
pub async fn run(ctx: SignalContext, signal: Signal<Artifact>) -> Result<(), Error> {
    let bytes = match &signal.content.content {
        nim::state::ArtifactContent::Text { text } => text.as_bytes().to_vec(),
        nim::state::ArtifactContent::File {
            path,
            size: _,
            media_type: _,
        } => tokio::fs::read(path).await?.to_vec(),
    };

    let captions = nim::ai::pipelines::images::captions::run(&[bytes])?;
    let Some(caption) = captions.first() else {
        return Err(Error::custom(format!(
            "no captions could be generated from artifact `{}`",
            signal.content.id
        )));
    };

    let memory = if let Some(embedding) = caption.vector.as_deref() {
        ctx.services()
            .memories()
            .search(embedding, nim::state::MemoryType::Working, 10)
            .await?
            .first()
            .cloned()
            .map(|v| v.into_inner())
    } else {
        None
    };

    let memory = if let Some(v) = memory {
        v
    } else {
        ctx.services()
            .memories()
            .create(nim::state::MemoryCreate {
                ty: nim::state::MemoryType::Working,
                sources: &[nim::state::Source::Region(nim::state::Region::Vision)],
                summary: caption.value.as_str(),
                ..Default::default()
            })
            .await?
    };

    ctx.services().artifacts().link(memory.id, signal.content.id).await?;
    Ok(())
}
