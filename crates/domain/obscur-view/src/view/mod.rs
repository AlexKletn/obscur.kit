//! View module.
mod view;
pub use view::View;
mod view_event_binding;
pub use view_event_binding::ViewEventBinding;
mod view_fragment;
pub use view_fragment::ViewFragment;
mod view_key;
pub use view_key::ViewKey;
mod view_node;
pub use view_node::ViewNode;
mod event_name;
pub use event_name::{EVENT_CHANGE, EVENT_CLICK, EVENT_INPUT};
mod slot;
pub use slot::{
    SLOT, SLOT_OUTLET, assigns_slots, has_outlet, is_outlet, project_slots, slot_assignment,
};
mod stable_selector;
pub use stable_selector::{AUTOMATION_ID, TEST_ID};
mod style_attribute;
pub use style_attribute::{
    CHECKED, CLASS, DISABLED, ID, PLACEHOLDER, READONLY, SRC, StyleAttributes, TYPE, VALUE,
};
mod view_property;
pub use view_property::ViewProperty;
mod view_value;
pub use view_value::ViewValue;
