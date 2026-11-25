//! What a display list did with images.

use obscur_geometry::Rect;
use obscur_scene::command::{DisplayList, PaintCommand};

#[doc = include_str!("../../docs/api/assertion/painted_images.doc/PaintedImages.md")]
#[derive(Debug)]
pub struct PaintedImages;

impl PaintedImages {
    #[doc = include_str!("../../docs/api/assertion/painted_images.doc/count.md")]
    #[must_use]
    pub fn count(list: &DisplayList) -> usize {
        list.commands()
            .iter()
            .filter(|c| matches!(c, PaintCommand::DrawImage { .. }))
            .count()
    }

    #[doc = include_str!("../../docs/api/assertion/painted_images.doc/tile.md")]
    #[must_use]
    pub fn tile(list: &DisplayList) -> Rect {
        list.commands()
            .iter()
            .find_map(|c| match c {
                PaintCommand::DrawImage { tile, .. } => Some(*tile),
                _ => None,
            })
            .expect("one DrawImage was painted")
    }
}
