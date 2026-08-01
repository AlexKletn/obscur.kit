#![doc = include_str!("mod.doc/_module.md")]
mod chrome_mode;
pub use chrome_mode::ChromeMode;
mod drag_region;
pub use drag_region::{DoubleClickAction, DragRegion};
mod resize_region;
pub use resize_region::{ResizeEdge, ResizeRegion};
mod system_menu_request;
pub use system_menu_request::SystemMenuRequest;
