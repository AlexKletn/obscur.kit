//! Pure winit→domain event converters (spec §6.1 windowing; §2.7 "преобразует
//! native events в доменные события"). Each converter is a stateless,
//! deterministic function of its winit input — no event-loop state — so the
//! §5.2/§6.1 determinism boundary holds.

mod display_converter;
pub use display_converter::DisplayConverter;
mod keyboard_converter;
pub use keyboard_converter::KeyboardConverter;
mod pointer_converter;
pub use pointer_converter::PointerConverter;
mod window_event_converter;
pub use window_event_converter::WindowEventConverter;
