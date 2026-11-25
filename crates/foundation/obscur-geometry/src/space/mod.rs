//! Space module.
mod local_space;
pub use local_space::LocalSpace;
mod parent_space;
pub use parent_space::ParentSpace;
mod physical_space;
pub use physical_space::PhysicalSpace;
mod screen_space;
pub use screen_space::ScreenSpace;
mod window_space;
pub use window_space::WindowSpace;
mod normalized_space;
pub use normalized_space::NormalizedSpace;
mod point_in;
pub use point_in::PointIn;
mod rect_in;
pub use rect_in::RectIn;
mod world_space;
pub use world_space::WorldSpace;
