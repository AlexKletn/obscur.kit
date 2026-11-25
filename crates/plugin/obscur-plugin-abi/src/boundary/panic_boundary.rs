//! Converts unwinding Rust panics to ABI failures inside the plugin.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::value::AbiStatus;

/// Converts unwinding Rust panics to ABI failures inside the plugin.
#[derive(Debug)]
pub struct PanicBoundary;

impl PanicBoundary {
    /// Converts an unwinding panic from Rust plugin code to an ABI status.
    pub fn status(callback: impl FnOnce() -> AbiStatus) -> AbiStatus {
        match catch_unwind(AssertUnwindSafe(callback)) {
            Ok(status) => status,
            Err(payload) => {
                // A custom panic payload can panic again when dropped.
                if let Err(drop_panic) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
                    std::mem::forget(drop_panic);
                }
                AbiStatus::PANICKED
            }
        }
    }
}
