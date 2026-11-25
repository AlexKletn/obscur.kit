//! Clipboard module.
mod clipboard_provider;
pub use clipboard_provider::ClipboardProvider;
mod memory_clipboard;
pub use memory_clipboard::MemoryClipboard;
mod clipboard_request;
pub use clipboard_request::ClipboardRequest;
mod clipboard_write_mode;
pub use clipboard_write_mode::ClipboardWriteMode;
mod clipboard_result;
pub use clipboard_result::ClipboardResult;
