//! Ime module.
mod composition;
pub use composition::Composition;
mod composition_state;
pub use composition_state::CompositionState;
mod composition_event;
pub use composition_event::CompositionEvent;
mod input_hints;
pub use input_hints::InputHints;
mod capitalization;
pub use capitalization::Capitalization;
mod input_purpose;
pub use input_purpose::InputPurpose;
mod preedit;
pub use preedit::Preedit;

mod text_input_update;
pub use text_input_update::TextInputUpdate;
