//! Property module.
mod custom_property;
pub use custom_property::CustomProperty;
mod inheritance;
pub use inheritance::Inheritance;
mod inherited_property;
pub use inherited_property::InheritedProperty;
pub mod layout_keyword;
mod length;
pub use length::Length;
mod property_id;
pub use property_id::PropertyId;
mod property_value;
pub use property_value::PropertyValue;
mod style_color;
pub use style_color::StyleColor;
