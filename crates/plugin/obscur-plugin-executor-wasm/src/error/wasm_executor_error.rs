//! Structured WebAssembly executor failures.

use std::fmt;

/// Errors produced while validating, loading, or invoking a WASM module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmExecutorError {
    /// The package does not declare a WASM plugin.
    WrongPluginKind,
    /// The entrypoint exceeded the module byte budget.
    ModuleTooLarge,
    /// The entrypoint failed binary module validation.
    InvalidModule(String),
    /// The module requests functions or data from a host environment.
    ImportsUnsupported,
    /// The module could not be instantiated under the configured limits.
    Instantiation(String),
    /// The requested function is absent or has the wrong signature.
    InvalidExport,
    /// The guest did not export a valid version-1 message buffer.
    InvalidMessageBuffer,
    /// The request exceeds the client budget or guest buffer capacity.
    RequestTooLarge,
    /// The guest returned a negative length or one beyond its buffer.
    InvalidResponseLength,
    /// The guest response exceeds the host reply budget.
    ResponseTooLarge,
    /// Guest execution consumed the fuel budget.
    OutOfFuel,
    /// Guest execution was stopped at a deadline check.
    DeadlineExceeded,
    /// Guest memory or table growth was denied.
    GrowthDenied,
    /// The module trapped while executing.
    Trap(String),
    /// The previous invocation trapped. Load a fresh instance to recover.
    Faulted,
    /// The managed instance is not active.
    Inactive,
    /// The instance is already processing a host call.
    Busy,
    /// Take the previously submitted host call before entering the guest again.
    HostCallPending,
}

impl fmt::Display for WasmExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPluginKind => f.write_str("package is not a WASM plugin"),
            Self::ModuleTooLarge => f.write_str("WASM entrypoint exceeds the module byte budget"),
            Self::InvalidModule(error) => write!(f, "invalid WASM module: {error}"),
            Self::ImportsUnsupported => f.write_str("WASM imports are not available"),
            Self::Instantiation(error) => write!(f, "WASM instantiation failed: {error}"),
            Self::InvalidExport => f.write_str("WASM export is missing or has the wrong signature"),
            Self::InvalidMessageBuffer => f.write_str("invalid WASM message buffer"),
            Self::RequestTooLarge => f.write_str("WASM request exceeds its byte budget"),
            Self::InvalidResponseLength => f.write_str("invalid WASM response length"),
            Self::ResponseTooLarge => f.write_str("WASM response exceeds its byte budget"),
            Self::OutOfFuel => f.write_str("WASM execution ran out of fuel"),
            Self::DeadlineExceeded => f.write_str("WASM execution reached its deadline"),
            Self::GrowthDenied => f.write_str("WASM memory or table growth was denied"),
            Self::Trap(error) => write!(f, "WASM execution trapped: {error}"),
            Self::Faulted => f.write_str("WASM instance requires a restart"),
            Self::Inactive => f.write_str("managed WASM instance is inactive"),
            Self::Busy => f.write_str("managed WASM instance is already processing a call"),
            Self::HostCallPending => f.write_str("WASM instance has a pending host call"),
        }
    }
}

impl std::error::Error for WasmExecutorError {}
