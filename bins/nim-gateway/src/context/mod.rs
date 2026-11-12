use crate::{Config, Error, Sockets};

pub mod http;

#[derive(Clone)]
pub struct Context {
    config: Config,
    pool: std::sync::Arc<sqlx::PgPool>,
    signals: std::sync::Arc<nim::signals::Client>,
    sockets: Sockets,
}

impl Context {
    pub fn services(&self) -> nim::state::Services<'_> {
        nim::state::Services::new(&self.pool)
    }

    pub fn signals(&self) -> &nim::signals::Client {
        &self.signals
    }

    pub fn runtime(&self) -> nim::runtime::Runtime<'_> {
        nim::runtime::Runtime::new(&self.pool, self.signals.clone())
    }

    pub fn sockets(&self) -> &Sockets {
        &self.sockets
    }
}

impl Context {
    #[tracing::instrument]
    pub async fn connect(config: Config) -> Result<Self, Error> {
        let pool = std::sync::Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?,
        );

        sqlx::migrate!("../../crates/nim-store/migrations").run(pool.as_ref()).await?;
        let signals = nim::signals::connect_with(
            "gateway",
            &config.nats_url,
            nim::signals::ConnectOptions {
                streams: vec![
                    nim::signals::StreamConfig {
                        name: "ARTIFACTS".into(),
                        max_messages: 10_000,
                        subjects: vec![
                            "nim.artifact.create".into(),
                            "nim.artifact.update".into(),
                            "nim.artifact.analyze".into(),
                            "nim.vision.artifact.create".into(),
                            "nim.audio.artifact.create".into(),
                            "nim.text.artifact.create".into(),
                            "nim.unknown.artifact.create".into(),
                        ],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "MEMORIES".into(),
                        max_messages: 10_000,
                        subjects: vec![
                            "nim.memory.create".into(),
                            "nim.memory.update".into(),
                            "nim.memory.analyze".into(),
                            "nim.memory.encode".into(),
                            "nim.memory.delete".into(),
                            "nim.memory.recall".into(),
                        ],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "ENTITIES".into(),
                        max_messages: 10_000,
                        subjects: vec![
                            "nim.entity.create".into(),
                            "nim.entity.update".into(),
                            "nim.entity.delete".into(),
                        ],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "TASKS".into(),
                        max_messages: 10_000,
                        subjects: vec![
                            "nim.task.create".into(),
                            "nim.task.start".into(),
                            "nim.task.complete".into(),
                            "nim.task.error".into(),
                        ],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "CHATS".into(),
                        max_messages: 10_000,
                        subjects: vec!["nim.chat.create".into(), "nim.chat.update".into(), "nim.chat.delete".into()],
                        ..Default::default()
                    },
                    nim::signals::StreamConfig {
                        name: "MESSAGES".into(),
                        max_messages: 10_000,
                        subjects: vec![
                            "nim.message.create".into(),
                            "nim.message.update".into(),
                            "nim.message.analyze".into(),
                            "nim.message.delete".into(),
                        ],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        )
        .await?;

        let signals = std::sync::Arc::new(signals);

        Ok(Self {
            config,
            pool,
            signals,
            sockets: Sockets::default(),
        })
    }
}

impl std::ops::Deref for Context {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}
