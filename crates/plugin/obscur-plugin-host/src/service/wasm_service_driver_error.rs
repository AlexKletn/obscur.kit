use std::{fmt, io};

use obscur_plugin_executor_wasm::error::WasmExecutorError;

/// Failure while driving a managed WASM host-service exchange.
#[derive(Debug)]
pub enum WasmServiceDriverError {
    /// The session belongs to a different plugin ID.
    PrincipalMismatch,
    /// The session has been revoked or an earlier delivery was interrupted.
    SessionRevoked,
    /// The guest instance rejected an operation.
    Guest(WasmExecutorError),
    /// The request or reply could not be framed.
    Packet(io::Error),
}

impl fmt::Display for WasmServiceDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrincipalMismatch => f.write_str("WASM service session principal mismatch"),
            Self::SessionRevoked => f.write_str("WASM service session revoked"),
            Self::Guest(error) => error.fmt(f),
            Self::Packet(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for WasmServiceDriverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Guest(error) => Some(error),
            Self::Packet(error) => Some(error),
            Self::PrincipalMismatch | Self::SessionRevoked => None,
        }
    }
}
