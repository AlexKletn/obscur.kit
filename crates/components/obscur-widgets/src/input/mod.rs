//! Text-input control (spec §2.6, §5.11).

mod input;
pub use input::Input;
mod input_component;
pub use input_component::InputComponent;
mod input_factory;
pub use input_factory::InputFactory;
mod input_props;
pub use input_props::InputProps;
