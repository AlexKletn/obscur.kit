//! Component module: the runtime's live component instances (spec §2.1, §2.14).
mod mounted_component;
pub use mounted_component::MountedComponent;
mod component_store;
pub use component_store::ComponentStore;
