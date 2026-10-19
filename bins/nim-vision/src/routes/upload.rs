use axum::response::IntoResponse;
use nim::state::Span;

use crate::http::HttpContext;

pub async fn route(
    ctx: HttpContext,
    mut multipart: axum::extract::Multipart,
) -> axum::response::Result<axum::response::Response> {
    let mut images = vec![];

    while let Some(field) = multipart.next_field().await? {
        let content_type = field.content_type().unwrap_or("application/octet-stream");

        if !content_type.starts_with("image/") {
            return Err((axum::http::StatusCode::BAD_REQUEST, "file must be an image".to_string()).into());
        }

        let bytes = field.bytes().await?;
        images.push(bytes.to_vec());
    }

    let mut memories = vec![];
    let artifacts = nim::ai::pipelines::images::captions::run(&images)
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    for artifact in &artifacts {
        let memory = ctx
            .services()
            .memories()
            .create(nim::state::MemoryCreate {
                ty: nim::state::MemoryType::Working,
                sources: &[nim::state::Source::Region(nim::state::Region::Vision)],
                summary: artifact.value.as_str(),
                embedding: artifact.vector.as_deref(),
                ..Default::default()
            })
            .await
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        memories.push(memory);
    }

    let annotations = nim::ai::pipelines::images::classify::run(&images)
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    for (memory, annotations) in memories.iter_mut().zip(annotations) {
        let texts = annotations.iter().map(|a| a.text.as_str()).collect::<Vec<_>>();
        let embeddings = nim::ai::pipelines::text::embeddings::run(&texts)
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        for (i, annotation) in annotations.iter().filter(|a| a.score >= 0.25).enumerate() {
            let annotation = ctx
                .services()
                .annotations()
                .create(nim::state::AnnotationCreate {
                    memory_id: memory.id,
                    label: nim::state::Label::Unknown,
                    text: annotation.label.clone(),
                    spans: &annotation
                        .spans
                        .iter()
                        .map(|v| Span {
                            start: v.begin,
                            end: v.end,
                        })
                        .collect::<Vec<_>>(),
                    confidence: annotation.score as f32,
                    embedding: embeddings[i].vector.as_deref(),
                })
                .await
                .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

            memory.annotations.push(annotation.into());
        }
    }

    let detections = nim::ai::pipelines::images::detect::run(&images)
        .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    for (memory, detections) in memories.iter_mut().zip(detections) {
        let texts = detections.iter().map(|a| a.label.as_str()).collect::<Vec<_>>();
        let embeddings = nim::ai::pipelines::text::embeddings::run(&texts)
            .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        for (i, detection) in detections.iter().filter(|a| a.score >= 0.25).enumerate() {
            let annotation = ctx
                .services()
                .annotations()
                .create(nim::state::AnnotationCreate {
                    memory_id: memory.id,
                    label: nim::state::Label::Unknown,
                    text: detection.label.clone(),
                    spans: Default::default(),
                    confidence: detection.score as f32,
                    embedding: embeddings[i].vector.as_deref(),
                })
                .await
                .map_err(|err| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

            memory.annotations.push(annotation.into());
        }
    }

    Ok(axum::Json(memories).into_response())
}
