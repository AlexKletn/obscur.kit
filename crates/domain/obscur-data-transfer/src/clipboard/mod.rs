//! Clipboard module.
mod clipboard_provider;
pub use clipboard_provider::{ClipboardProvider, MemoryClipboard};
mod clipboard_request;
pub use clipboard_request::{ClipboardRequest, ClipboardWriteMode};
mod clipboard_result;
pub use clipboard_result::ClipboardResult;
