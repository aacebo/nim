use std::sync::Arc;

use crate::Source;

pub struct Diagnostic {
    _span: tracing::Span,
    _source: Source,
    _location: &'static std::panic::Location<'static>,
    _backtrace: std::backtrace::Backtrace,
    _children: Vec<Arc<dyn Send + Sync + std::error::Error>>,
}

impl Diagnostic {
    #[track_caller]
    pub fn message<T: std::fmt::Display>(message: T) -> Self {
        Self {
            _span: tracing::Span::current(),
            _source: Source::Message(message.to_string()),
            _location: std::panic::Location::caller(),
            _backtrace: std::backtrace::Backtrace::capture(),
            _children: Vec::new(),
        }
    }

    #[track_caller]
    pub fn error<T: Send + Sync + std::error::Error + 'static>(error: T) -> Self {
        Self {
            _span: tracing::Span::current(),
            _source: Source::Error(Arc::new(error)),
            _location: std::panic::Location::caller(),
            _backtrace: std::backtrace::Backtrace::capture(),
            _children: Vec::new(),
        }
    }

    pub fn child<T: Send + Sync + std::error::Error + 'static>(mut self, error: T) -> Self {
        self._children.push(Arc::new(error));
        self
    }
}

impl Diagnostic {
    pub fn span(&self) -> &tracing::Span {
        &self._span
    }

    pub fn source(&self) -> &Source {
        &self._source
    }

    pub fn location(&self) -> &std::panic::Location<'_> {
        self._location
    }

    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        &self._backtrace
    }

    pub fn children(&self) -> &[Arc<dyn Send + Sync + std::error::Error>] {
        &self._children
    }
}

impl IntoIterator for Diagnostic {
    type Item = Arc<dyn Send + Sync + std::error::Error + 'static>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self._children.into_iter()
    }
}

impl std::fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Diagnostic")
            .field("span", &self._span)
            .field("source", &self._source)
            .field("location", self._location)
            .field("backtrace", &self._backtrace)
            .field("children", &self._children)
            .finish()
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (headline_style, location_style, cause_style, trace_style, reset) = {
            if std::env::var("NIM_ERROR_COLOR").as_deref() == Ok("true") {
                let headline = anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Red.into())).bold();
                let location = anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Cyan.into()));
                let cause = anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::Yellow.into()));
                let trace = anstyle::Style::new().dimmed();
                (
                    headline.to_string(),
                    location.to_string(),
                    cause.to_string(),
                    trace.to_string(),
                    headline.render_reset().to_string(),
                )
            } else {
                (String::new(), String::new(), String::new(), String::new(), String::new())
            }
        };

        let message = self._source.to_string();
        let headline = message.lines().next().unwrap_or("error");
        writeln!(f, "{headline_style}× {headline}{reset}")?;

        for line in message.lines().skip(1) {
            writeln!(f, "├─▶ {cause_style}{line}{reset}")?;
        }

        let child_count = self._children.len();
        for (index, error) in self._children.iter().enumerate() {
            let branch = if index + 1 == child_count { "╰─▶" } else { "├─▶" };
            let mut current: Option<&(dyn std::error::Error + 'static)> = Some(error.as_ref());
            let mut first = true;

            while let Some(cause) = current {
                let prefix = if first { branch } else { "│   ╰─▶" };
                writeln!(f, "{prefix} {cause_style}{cause}{reset}")?;
                current = cause.source();
                first = false;
            }

            if index + 1 != child_count {
                writeln!(f, "│")?;
            }
        }

        writeln!(
            f,
            "{location_style}╭─[{}:{}:{}]{reset}",
            self._location.file(),
            self._location.line(),
            self._location.column(),
        )?;

        writeln!(f, "│")?;

        if self._backtrace.status() == std::backtrace::BacktraceStatus::Captured {
            writeln!(f, "╰─ stack trace:")?;

            for line in self._backtrace.to_string().lines() {
                writeln!(f, "  {trace_style}{line}{reset}")?;
            }
        } else {
            writeln!(f, "╰─")?;
        }

        Ok(())
    }
}

impl<T: Send + Sync + std::error::Error + 'static> From<T> for Diagnostic {
    fn from(value: T) -> Self {
        Self::error(value)
    }
}
