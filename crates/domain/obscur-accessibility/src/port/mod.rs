#![doc = include_str!("mod.doc/_module.md")]
mod accessibility_bridge;
pub use accessibility_bridge::{
    AccessibilityBridge, NullAccessibilityBridge, RecordingAccessibilityBridge,
};
