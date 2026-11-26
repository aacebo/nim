use std::sync::Arc;

use crate::{IntoDiagnostic, Source};

pub struct Diagnostic {
    _span: tracing::Span,
    _source: Source,
    _location: &'static std::panic::Location<'static>,
    _backtrace: std::backtrace::Backtrace,
    _fields: serde_json::Map<String, serde_json::Value>,
    _children: Vec<Self>,
}

impl Diagnostic {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            _span: tracing::Span::current(),
            _source: Source::None,
            _location: std::panic::Location::caller(),
            _backtrace: std::backtrace::Backtrace::capture(),
            _fields: Default::default(),
            _children: Vec::new(),
        }
    }

    #[track_caller]
    pub fn message<T: std::fmt::Display>(message: T) -> Self {
        Self {
            _span: tracing::Span::current(),
            _source: Source::Message(message.to_string()),
            _location: std::panic::Location::caller(),
            _backtrace: std::backtrace::Backtrace::capture(),
            _fields: Default::default(),
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
            _fields: Default::default(),
            _children: Vec::new(),
        }
    }

    pub fn child<T: IntoDiagnostic>(mut self, error: T) -> Self {
        self._children.push(error.into_diagnostic());
        self
    }

    pub fn append<T: IntoDiagnostic>(mut self, errors: impl IntoIterator<Item = T>) -> Self {
        self._children.extend(errors.into_iter().map(|v| v.into_diagnostic()));
        self
    }

    pub fn field<K: ToString, V: serde::Serialize>(mut self, name: K, value: V) -> Self {
        match serde_json::to_value(value) {
            Err(err) => {
                self._children.push(Self::error(err));
                self
            }
            Ok(v) => {
                self._fields.insert(name.to_string(), v);
                self
            }
        }
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

    pub fn get(&self, key: impl AsRef<str>) -> Option<&serde_json::Value> {
        self._fields.get(key.as_ref())
    }

    pub fn children(&self) -> &[Self] {
        &self._children
    }
}

impl<T: Send + Sync + std::error::Error + 'static> From<T> for Diagnostic {
    fn from(value: T) -> Self {
        Self::error(value)
    }
}

impl IntoIterator for Diagnostic {
    type Item = Self;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self._children.into_iter()
    }
}

impl<V: IntoDiagnostic> FromIterator<V> for Diagnostic {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut diagnostic = Self::new();
        diagnostic._children.extend(iter.into_iter().map(|v| v.into_diagnostic()));
        diagnostic
    }
}

impl std::fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Diagnostic")
            .field("span", &self._span)
            .field("source", &self._source)
            .field("location", self._location)
            .field("backtrace", &self._backtrace)
            .field("fields", &self._fields)
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

        render(f, self, "", &cause_style, &reset)?;

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

fn render(f: &mut std::fmt::Formatter<'_>, diagnostic: &Diagnostic, prefix: &str, style: &str, reset: &str) -> std::fmt::Result {
    let cause = match &diagnostic._source {
        Source::Error(error) => error.source(),
        _ => None,
    };
    let count = diagnostic._fields.len() + diagnostic._children.len() + usize::from(cause.is_some());
    let mut index = 0;

    for (name, value) in &diagnostic._fields {
        index += 1;
        let branch = if index == count { "╰─▶" } else { "├─▶" };
        writeln!(f, "{prefix}{branch} {style}{name}: {value}{reset}")?;
    }

    if let Some(mut cause) = cause {
        index += 1;
        let branch = if index == count { "╰─▶" } else { "├─▶" };
        let mut indent = format!("{prefix}{}", if index == count { "    " } else { "│   " });

        writeln!(f, "{prefix}{branch} {style}{cause}{reset}")?;

        while let Some(next) = cause.source() {
            writeln!(f, "{indent}╰─▶ {style}{next}{reset}")?;
            indent.push_str("    ");
            cause = next;
        }
    }

    for child in &diagnostic._children {
        index += 1;
        let last = index == count;
        let branch = if last { "╰─▶" } else { "├─▶" };
        let indent = format!("{prefix}{}", if last { "    " } else { "│   " });
        let message = child._source.to_string();
        let mut lines = message.lines();

        writeln!(f, "{prefix}{branch} {style}{}{reset}", lines.next().unwrap_or("error"))?;

        for line in lines {
            writeln!(f, "{indent}    {style}{line}{reset}")?;
        }

        render(f, child, &indent, style, reset)?;
    }

    Ok(())
}
