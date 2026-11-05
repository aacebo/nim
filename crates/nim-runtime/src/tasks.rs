use nim_state::{ArtifactId, ChatId, MessageId, ModelId, TaskId};

#[derive(Debug, Default, Clone)]
pub struct TaskOptions {
    pub parent_id: Option<TaskId>,
    pub sources: Vec<TaskSource>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskSource {
    Artifact(ArtifactId),
    Chat(ChatId),
    Message(MessageId),
    Model(ModelId),
}

impl From<ArtifactId> for TaskSource {
    fn from(value: ArtifactId) -> Self {
        Self::Artifact(value)
    }
}

impl From<ChatId> for TaskSource {
    fn from(value: ChatId) -> Self {
        Self::Chat(value)
    }
}

impl From<MessageId> for TaskSource {
    fn from(value: MessageId) -> Self {
        Self::Message(value)
    }
}

impl From<ModelId> for TaskSource {
    fn from(value: ModelId) -> Self {
        Self::Model(value)
    }
}

macro_rules! task_run {
    ($runtime:expr, $name:expr, $options:expr, $run:block) => {{
        let services = &$runtime._services;
        let signals = &$runtime._signals;
        let task = $crate::tasks::start(services, signals, $name, &$options.task).await?;
        let result: Result<_, $crate::Error> = async {
            let output = $run?;
            Ok(output)
        }
        .await;

        $crate::tasks::complete(services, signals, task, result).await
    }};
}

pub(crate) async fn start<'a>(
    services: &nim_state::Services<'a>,
    signals: &nim_signals::Client,
    name: impl std::fmt::Display,
    options: &TaskOptions,
) -> Result<nim_state::Task, Error> {
    let task = services
        .tasks()
        .create(nim_state::TaskCreate {
            parent_id: options.parent_id,
            name: name.to_string(),
            source: nim_state::Source::System,
        })
        .await?;

    for source in &options.sources {
        match source {
            TaskSource::Artifact(id) => {
                services.tasks().link_artifact(*id, task.id).await?;
            }
            TaskSource::Chat(id) => {
                services.tasks().link_chat(*id, task.id).await?;
            }
            TaskSource::Message(id) => {
                services.tasks().link_message(*id, task.id).await?;
            }
            TaskSource::Model(id) => {
                services.tasks().link_model(*id, task.id).await?;
            }
        };
    }

    signals.publish("nim.task.create", task.clone()).await?;

    let started_at = chrono::Utc::now();
    let task = services
        .tasks()
        .update(
            task.id,
            nim_state::TaskUpdate {
                status: Some(nim_state::TaskStatus::Started {
                    output: None,
                    started_at,
                }),
                ..Default::default()
            },
        )
        .await?;

    if let Err(err) = signals.publish("nim.task.start", task.clone()).await {
        if let Ok(task) = services
            .tasks()
            .update(
                task.id,
                nim_state::TaskUpdate {
                    status: Some(nim_state::TaskStatus::Error {
                        reason: err.to_string(),
                        started_at,
                        ended_at: chrono::Utc::now(),
                    }),
                    ..Default::default()
                },
            )
            .await
        {
            let _ = signals.publish("nim.task.error", task).await;
        }

        return Err(err.into());
    }
    Ok(task)
}

pub(crate) async fn complete<'a, T>(
    services: &nim_state::Services<'a>,
    signals: &nim_signals::Client,
    task: nim_state::Task,
    result: Result<T, Error>,
) -> Result<T, Error> {
    let (output, started_at) = match &task.status {
        nim_state::TaskStatus::Started { output, started_at } => (output.clone(), *started_at),
        _ => return Err(Error::custom("invalid task state")),
    };

    match result {
        Err(err) => {
            let task = services
                .tasks()
                .update(
                    task.id,
                    nim_state::TaskUpdate {
                        status: Some(nim_state::TaskStatus::Error {
                            reason: err.to_string(),
                            started_at,
                            ended_at: chrono::Utc::now(),
                        }),
                        ..Default::default()
                    },
                )
                .await?;

            signals.publish("nim.task.error", task.clone()).await?;
            Err(err)
        }
        Ok(value) => {
            let task = services
                .tasks()
                .update(
                    task.id,
                    nim_state::TaskUpdate {
                        status: Some(nim_state::TaskStatus::Complete {
                            output,
                            started_at,
                            ended_at: chrono::Utc::now(),
                        }),
                        ..Default::default()
                    },
                )
                .await?;

            signals.publish("nim.task.complete", task.clone()).await?;
            Ok(value)
        }
    }
}

pub(crate) use task_run;

use crate::Error;
