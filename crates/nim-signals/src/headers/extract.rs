use async_nats::HeaderMap;
use opentelemetry::propagation::Extractor;

pub struct NatsExtractor<'a>(&'a HeaderMap);

impl<'a> From<&'a HeaderMap> for NatsExtractor<'a> {
    fn from(value: &'a HeaderMap) -> Self {
        Self(value)
    }
}

impl Extractor for NatsExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|value| value.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|(key, _)| key.as_ref()).collect()
    }
}
