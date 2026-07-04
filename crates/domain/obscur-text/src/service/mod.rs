//! Service module.
mod language_service_request;
pub use language_service_request::{LanguageServiceRequest, ServiceKind};
mod text_diagnostic;
pub use text_diagnostic::{DiagnosticKind, DiagnosticSeverity, TextDiagnostic};
mod text_service_provider;
pub use text_service_provider::{
    MockTextServiceProvider, NullTextServiceProvider, TextServiceProvider,
};
