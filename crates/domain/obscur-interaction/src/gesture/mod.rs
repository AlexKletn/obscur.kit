#![doc = include_str!("mod.doc/_module.md")]
mod gesture_arena;
pub use gesture_arena::GestureArena;
mod gesture_contender;
pub use gesture_contender::GestureContender;
mod gesture_recognizer;
pub use gesture_recognizer::{GestureKind, GestureRecognizer, RecognizerId, RecognizerState};
mod gesture_resolution;
pub use gesture_resolution::GestureResolution;
mod fling;
pub use fling::Fling;
mod long_press_recognizer;
pub use long_press_recognizer::LongPressRecognizer;
mod pan_recognizer;
pub use pan_recognizer::PanRecognizer;
