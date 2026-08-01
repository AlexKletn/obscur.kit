#![doc = include_str!("mod.doc/_module.md")]
mod composition;
pub use composition::{Composition, CompositionState};
mod composition_event;
pub use composition_event::CompositionEvent;
mod input_hints;
pub use input_hints::{Capitalization, InputHints};
mod input_purpose;
pub use input_purpose::InputPurpose;
mod preedit;
pub use preedit::Preedit;
