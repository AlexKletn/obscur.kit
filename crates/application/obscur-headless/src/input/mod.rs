//! Injectable typed synthetic-input descriptors for headless tests (spec §7.4, §7.5).

mod pointer_button;
pub use pointer_button::PointerButton;
mod selector_target;
pub use selector_target::SelectorTarget;
mod synthetic_input;
pub use synthetic_input::SyntheticInput;
