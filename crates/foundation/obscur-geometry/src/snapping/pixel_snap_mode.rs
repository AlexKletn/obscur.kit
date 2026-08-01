#![doc = include_str!("pixel_snap_mode.doc/_module.md")]

#[doc = include_str!("pixel_snap_mode.doc/PixelSnapMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSnapMode {
    #[doc = include_str!("pixel_snap_mode.doc/Auto.md")]
    #[default]
    Auto,
    #[doc = include_str!("pixel_snap_mode.doc/None.md")]
    None,
    #[doc = include_str!("pixel_snap_mode.doc/Edges.md")]
    Edges,
}
