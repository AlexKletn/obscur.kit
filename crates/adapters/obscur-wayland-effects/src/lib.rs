//! Wayland compositor effects for existing client surfaces.
#![deny(unsafe_op_in_unsafe_fn)]

mod background_blur;
pub use background_blur::BackgroundBlur;
mod rounded_region;
mod window_blur;
pub use window_blur::WindowBlur;
mod window_presentation;
pub use window_presentation::WindowPresentation;
mod window_gestures;
pub use window_gestures::WindowGestures;

mod pointing_capabilities;
pub use pointing_capabilities::WaylandPointingCapabilities;

mod presentation_feedback;
mod presentation_timestamp;
pub use presentation_timestamp::presentation_timestamp_micros;
