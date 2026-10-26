use std::time::{Duration, Instant};

pub(super) struct Download {
    repository: String,
    filename: String,
    downloaded: u64,
    total: Option<u64>,
    started: Instant,
    last: Instant,
}

impl Download {
    pub fn new(repository: &str, filename: &str) -> Self {
        let now = Instant::now();

        Self {
            repository: redact(repository),
            filename: redact(filename),
            downloaded: 0,
            total: None,
            started: now,
            last: now,
        }
    }

    pub fn start(&self) {
        tracing::info!(
            target: "nim_ai::clients::download",
            repository = %self.repository,
            filename = %self.filename,
            downloaded_bytes = self.downloaded,
            total_bytes = self.total,
            elapsed_seconds = self.started.elapsed().as_secs_f64(),
            "model download started",
        );
    }

    pub fn reset(&mut self, total: Option<u64>) {
        self.total = total;
        self.downloaded = 0;
    }

    pub fn update(&mut self, bytes: usize) {
        self.downloaded = self.downloaded.saturating_add(bytes as u64);

        if !tracing::enabled!(target: "nim_ai::clients::download", tracing::Level::INFO) {
            return;
        }

        let now = Instant::now();

        if now.duration_since(self.last) >= Duration::from_secs(1) {
            self.last = now;
            tracing::info!(
                target: "nim_ai::clients::download",
                repository = %self.repository,
                filename = %self.filename,
                downloaded_bytes = self.downloaded,
                total_bytes = self.total,
                percent = self.percent(),
                elapsed_seconds = self.started.elapsed().as_secs_f64(),
                "model download progress",
            );
        }
    }

    pub fn complete(&self) {
        tracing::info!(
            target: "nim_ai::clients::download",
            repository = %self.repository,
            filename = %self.filename,
            downloaded_bytes = self.downloaded,
            total_bytes = self.total,
            percent = self.percent(),
            elapsed_seconds = self.started.elapsed().as_secs_f64(),
            "model download complete",
        );
    }

    pub fn failed(&self) {
        tracing::debug!(
            target: "nim_ai::clients::download",
            repository = %self.repository,
            filename = %self.filename,
            downloaded_bytes = self.downloaded,
            total_bytes = self.total,
            elapsed_seconds = self.started.elapsed().as_secs_f64(),
            "model download failed",
        );
    }

    pub fn cached(&self) {
        tracing::debug!(
            target: "nim_ai::clients::download",
            repository = %self.repository,
            filename = %self.filename,
            "model cache hit",
        );
    }

    fn percent(&self) -> Option<f64> {
        self.total
            .filter(|total| *total > 0)
            .map(|total| self.downloaded as f64 / total as f64 * 100.0)
    }
}

fn redact(value: &str) -> String {
    match url::Url::parse(value) {
        Ok(mut url) => {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_query(None);
            url.set_fragment(None);
            url.to_string()
        }
        Err(_) => value.split(['?', '#']).next().unwrap_or_default().to_string(),
    }
}
