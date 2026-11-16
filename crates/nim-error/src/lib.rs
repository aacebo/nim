mod diagnostic;
mod source;

pub use diagnostic::*;
pub use source::*;

pub type Result<T> = std::result::Result<T, Diagnostic>;

pub trait IntoDiagnostic {
    fn into_diagnostic(self) -> Diagnostic;
}

impl<T: Send + Sync + std::error::Error + 'static> IntoDiagnostic for T {
    fn into_diagnostic(self) -> Diagnostic {
        Diagnostic::error(self)
    }
}
