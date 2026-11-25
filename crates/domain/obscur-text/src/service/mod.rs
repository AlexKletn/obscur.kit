//! Service module.
mod language_service_request;
pub use language_service_request::LanguageServiceRequest;
mod service_kind;
pub use service_kind::ServiceKind;
mod diagnostic_kind;
pub use diagnostic_kind::DiagnosticKind;
mod diagnostic_severity;
pub use diagnostic_severity::DiagnosticSeverity;
mod text_diagnostic;
pub use text_diagnostic::TextDiagnostic;
mod text_service_provider;
pub use text_service_provider::TextServiceProvider;
mod null_text_service_provider;
pub use null_text_service_provider::NullTextServiceProvider;
mod mock_text_service_provider;
pub use mock_text_service_provider::MockTextServiceProvider;
