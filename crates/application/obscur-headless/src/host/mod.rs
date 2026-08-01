#![doc = include_str!("mod.doc/_module.md")]
mod headless_host;

pub use headless_host::HeadlessHost;

mod headless_config;
pub use headless_config::HeadlessConfig;
mod virtual_window;
pub use virtual_window::VirtualWindow;
