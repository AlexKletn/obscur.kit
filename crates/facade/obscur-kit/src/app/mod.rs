#![doc = include_str!("mod.doc/_module.md")]
mod app;
pub use app::{App, FrameInfo};
mod app_error;
pub use app_error::AppError;
mod runner;
