//! Shared content alignment for flex lines and grid tracks.

use crate::model::JustifyContent;

/// Returns the leading offset and extra gutter. Center and end alignment use signed free space. Distributed gutters use only positive free space.
pub(super) fn content_distribution(n: usize, leftover: f32, mode: JustifyContent) -> (f32, f32) {
    if n == 0 || !leftover.is_finite() {
        return (0.0, 0.0);
    }
    let distributable = leftover.max(0.0);
    match mode {
        // `stretch` grows the tracks themselves to consume the leftover (handled by
        // the track-stretch step), so it leaves no gaps to distribute here.
        JustifyContent::Normal | JustifyContent::FlexStart | JustifyContent::Stretch => (0.0, 0.0),
        JustifyContent::FlexEnd => (leftover, 0.0),
        JustifyContent::Center => (leftover / 2.0, 0.0),
        JustifyContent::SpaceBetween => {
            if n >= 2 {
                (0.0, distributable / (n - 1) as f32)
            } else {
                (0.0, 0.0)
            }
        }
        JustifyContent::SpaceAround => {
            let unit = distributable / n as f32;
            (unit / 2.0, unit)
        }
        JustifyContent::SpaceEvenly => {
            let unit = distributable / (n + 1) as f32;
            (unit, unit)
        }
    }
}
