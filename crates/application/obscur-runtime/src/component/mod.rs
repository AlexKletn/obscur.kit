//! Component module: the runtime's live component instances (spec §2.1, §2.14).
mod guarded_render;
pub use guarded_render::GuardedRender;
mod mounted_component;
pub use mounted_component::{COMPONENT_PROPS_FAILED, MountedComponent};
mod component_store;
pub use component_store::ComponentStore;
