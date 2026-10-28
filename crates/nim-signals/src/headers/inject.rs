use async_nats::HeaderMap;
use opentelemetry::propagation::Injector;

pub struct NatsInjector<'a>(&'a mut HeaderMap);

impl<'a> From<&'a mut HeaderMap> for NatsInjector<'a> {
    fn from(value: &'a mut HeaderMap) -> Self {
        Self(value)
    }
}

impl Injector for NatsInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key, value);
    }
}
