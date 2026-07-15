//! Injectable typed synthetic-input descriptors for headless tests (spec §7.4, §7.5).

mod dispatch_outcome;
pub use dispatch_outcome::DispatchOutcome;
mod pointer_button;
pub use pointer_button::PointerButton;
mod selector_resolver;
pub use selector_resolver::SelectorResolver;
mod selector_target;
pub use selector_target::SelectorTarget;
mod synthetic_dispatcher;
pub use synthetic_dispatcher::SyntheticDispatcher;
mod synthetic_input;
pub use synthetic_input::SyntheticInput;
