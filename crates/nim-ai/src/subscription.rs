pub struct Subscription {
    cancel: Option<Box<dyn FnOnce() + Send + Sync + 'static>>,
}

impl Subscription {
    pub(crate) fn new(cancel: impl FnOnce() + Send + Sync + 'static) -> Self {
        Self {
            cancel: Some(Box::new(cancel)),
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel();
        }
    }
}
