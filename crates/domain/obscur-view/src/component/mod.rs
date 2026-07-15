//! Component module.
mod component_contract;

pub use component_contract::ComponentContract;

mod component;
pub use component::Component;
mod component_context;
pub use component_context::ComponentContext;
mod component_factory;
pub use component_factory::ComponentFactory;
mod component_instance;
pub use component_instance::ComponentInstance;
mod instance_id;
pub use instance_id::InstanceId;
mod component_version;
pub use component_version::ComponentVersion;
mod component_registration;
pub use component_registration::ComponentRegistration;
mod registry_error;
pub use registry_error::RegistryError;
mod component_registry;
pub use component_registry::ComponentRegistry;
mod props_error;
pub use props_error::PropsError;
mod from_view_value;
pub use from_view_value::FromViewValue;
mod from_props;
pub use from_props::{FromProps, decode_optional, decode_required, decode_value, prop_value};
mod erased_component;
pub use erased_component::ErasedComponent;
