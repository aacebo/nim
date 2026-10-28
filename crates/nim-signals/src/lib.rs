mod client;
mod endpoint;
mod error;
mod headers;
mod request;
mod service;
mod stream;
pub mod trace;
mod types;

pub use client::*;
pub use endpoint::*;
pub use error::*;
pub use request::*;
pub use service::*;
pub use stream::*;
pub use types::*;

pub use async_nats::ConnectOptions as NatsConnectOptions;
pub use async_nats::Error as NatsError;
pub use async_nats::jetstream::AckKind;
pub use async_nats::jetstream::Message;
pub use async_nats::jetstream::stream::Config as StreamConfig;
pub use async_nats::service::error::Error as ServiceError;

#[derive(Debug, Default)]
pub struct ConnectOptions {
    pub connect: NatsConnectOptions,
    pub streams: Vec<StreamConfig>,
}

#[tracing::instrument]
pub async fn connect(name: &str, addr: &str) -> Result<Client, Error> {
    connect_with(name, addr, Default::default()).await
}

#[tracing::instrument]
pub async fn connect_with(name: &str, addr: &str, options: ConnectOptions) -> Result<Client, Error> {
    let client = async_nats::connect_with_options(addr, options.connect).await?;
    let context = async_nats::jetstream::new(client);

    for stream in options.streams {
        context.create_or_update_stream(stream).await?;
    }

    Ok(Client::new(name, context))
}
