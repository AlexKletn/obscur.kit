#![doc = include_str!("mod.doc/_module.md")]
mod canvas_region;
pub use canvas_region::CanvasRegion;
mod display_list;
pub use display_list::DisplayList;
mod display_list_builder;
pub use display_list_builder::DisplayListBuilder;
mod paint_command;
pub use paint_command::PaintCommand;
