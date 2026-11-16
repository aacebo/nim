use std::sync::Arc;

use crate::Source;

pub struct Diagnostic {
    span: tracing::Span,
    source: Source,
    children: Vec<Arc<dyn Send + Sync + std::error::Error>>,
}

impl Diagnostic {
    pub fn message<T: std::fmt::Display>(message: T) -> Self {
        Self {
            span: tracing::Span::current(),
            source: Source::Message(message.to_string()),
            children: Vec::new(),
        }
    }

    pub fn error<T: Send + Sync + std::error::Error + 'static>(error: T) -> Self {
        Self {
            span: tracing::Span::current(),
            source: Source::Error(Arc::new(error)),
            children: Vec::new(),
        }
    }

    pub fn child<T: Send + Sync + std::error::Error + 'static>(mut self, error: T) -> Self {
        self.children.push(Arc::new(error));
        self
    }
}

impl Diagnostic {
    pub fn span(&self) -> &tracing::Span {
        &self.span
    }

    pub fn children(&self) -> &[Arc<dyn Send + Sync + std::error::Error>] {
        &self.children
    }
}

impl IntoIterator for Diagnostic {
    type Item = Arc<dyn Send + Sync + std::error::Error + 'static>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

impl std::fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Diagnostic")
            .field("span", &self.span)
            .field("source", &self.source)
            .field("children", &self.children)
            .finish()
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(meta) = self.span.metadata() {
            writeln!(f, "[error::{}]", meta.name())?;

            if let (Some(file), Some(line)) = (meta.file(), meta.line()) {
                writeln!(f, "\t@ {file} Ln {line}")?;
            }
        }

        if !self.source.is_none() {
            for line in self.source.to_string().lines() {
                writeln!(f, "\t{line}")?;
            }
        }

        for child in &self.children {
            for line in child.to_string().lines() {
                writeln!(f, "\t{line}")?;
            }
        }

        Ok(())
    }
}

impl<T: Send + Sync + std::error::Error + 'static> From<T> for Diagnostic {
    fn from(value: T) -> Self {
        Self::error(value)
    }
}
