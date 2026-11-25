//! Presentation module.
mod fullscreen_mode;
pub use fullscreen_mode::FullscreenMode;
mod presentation_mode;
pub use presentation_mode::PresentationMode;
mod safe_escape_policy;
pub use safe_escape_policy::SafeEscapePolicy;
mod sleep_inhibition;
pub use sleep_inhibition::SleepInhibition;
mod sleep_inhibition_scope;
pub use sleep_inhibition_scope::SleepInhibitionScope;
