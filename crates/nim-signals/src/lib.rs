mod client;
mod error;
mod stream;
mod types;

pub use client::*;
pub use error::*;
pub use stream::*;
pub use types::*;

#[derive(Debug, Default)]
pub struct ConnectOptions {
    pub name: String,
    pub connect: async_nats::ConnectOptions,
    pub streams: Vec<async_nats::jetstream::stream::Config>,
}

#[tracing::instrument]
pub async fn connect(name: &str, addr: &str) -> Result<Client, Error> {
    connect_with(name, addr, Default::default()).await
}

#[tracing::instrument]
pub async fn connect_with(name: &str, addr: &str, options: ConnectOptions) -> Result<Client, Error> {
    let client = async_nats::connect_with_options(addr, options.connect).await?;
    let context = async_nats::jetstream::new(client);
    Ok(Client::new(name, context))
}
