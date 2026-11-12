use std::{collections::HashMap, sync::Arc};

use tokio::sync::{RwLock, mpsc::UnboundedSender};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SocketSignal {
    pub subject: String,

    #[serde(flatten)]
    pub signal: nim::signals::RawSignal,
}

#[derive(Clone, Default)]
pub struct Sockets {
    inner: Arc<RwLock<HashMap<uuid::Uuid, UnboundedSender<SocketSignal>>>>,
}

impl Sockets {
    pub async fn open(&self) -> Subscription {
        let id = uuid::Uuid::now_v7();
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let mut map = self.inner.write().await;
        map.insert(id, sender);
        Subscription { id, receiver }
    }

    pub async fn close(&self, subscription: Subscription) {
        let mut map = self.inner.write().await;
        map.remove(&subscription.id);
    }

    pub async fn send(&self, subject: String, signal: nim::signals::RawSignal) -> Result<(), Box<dyn std::error::Error>> {
        let signal = SocketSignal { subject, signal };
        let mut map = self.inner.write().await;
        let mut dead = Vec::new();

        for (id, sender) in map.iter() {
            if sender.is_closed() {
                dead.push(*id);
                continue;
            };

            sender.send(signal.clone())?;
        }

        for id in &dead {
            map.remove(id);
        }

        Ok(())
    }
}

pub struct Subscription {
    id: uuid::Uuid,
    receiver: tokio::sync::mpsc::UnboundedReceiver<SocketSignal>,
}

impl std::ops::Deref for Subscription {
    type Target = tokio::sync::mpsc::UnboundedReceiver<SocketSignal>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}

impl std::ops::DerefMut for Subscription {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.receiver
    }
}
