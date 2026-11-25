use crate::lowering::LoweringError;
use crate::resolution::ResolutionError;
use crate::typing::TypeError;
use crate::validation::ValidationDiagnostic;

/// A single diagnostic produced by the compilation pipeline: the sum of every
/// pass's error type, so callers see one uniform stream (spec §2.9–§2.15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileDiagnostic {
    /// Name resolution failed for a reference or declaration (§2.10, §2.11).
    Resolution(ResolutionError),
    /// A type check failed (§2.9). Reserved for the future whole-document type
    /// pass; the orchestrator does not emit it yet (see [`Compiler::compile`]).
    Type(TypeError),
    /// A static validation rule was violated — an expression, capability or
    /// contract check (§2.10, §2.13).
    Validation(ValidationDiagnostic),
    /// Lowering to UI IR failed (§2.15).
    Lowering(LoweringError),
}

impl std::fmt::Display for CompileDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolution(error) => write!(f, "{error}"),
            Self::Type(error) => write!(f, "{error}"),
            Self::Validation(diagnostic) => {
                write!(
                    f,
                    "{} at offset {}",
                    diagnostic.message(),
                    diagnostic.offset()
                )
            }
            Self::Lowering(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CompileDiagnostic {}

impl From<ResolutionError> for CompileDiagnostic {
    fn from(error: ResolutionError) -> Self {
        Self::Resolution(error)
    }
}

impl From<TypeError> for CompileDiagnostic {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}

impl From<ValidationDiagnostic> for CompileDiagnostic {
    fn from(diagnostic: ValidationDiagnostic) -> Self {
        Self::Validation(diagnostic)
    }
}

impl From<LoweringError> for CompileDiagnostic {
    fn from(error: LoweringError) -> Self {
        Self::Lowering(error)
    }
}
