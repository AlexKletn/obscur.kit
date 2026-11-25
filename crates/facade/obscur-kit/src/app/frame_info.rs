use crate::geometry::Size;

#[doc = include_str!("../../docs/api/app/app.doc/FrameInfo.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameInfo {
    /// The viewport in logical (CSS) pixels.
    pub viewport: Size,
    /// The display's device-pixel ratio.
    pub scale: f32,
}
