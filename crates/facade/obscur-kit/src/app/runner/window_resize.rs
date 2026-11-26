//! Resize grips for undecorated application windows.

use winit::window::{CursorIcon, ResizeDirection};

use super::AppRunner;
use crate::geometry::{CornerRadii, Point, Size};

/// Width of the resize grip in logical pixels.
const EDGE_WIDTH: f32 = 6.0;

impl AppRunner {
    /// Find the active resize grip at the mouse position.
    pub(super) fn resize_direction(&self) -> Option<ResizeDirection> {
        // Winit does not support starting an interactive resize on macOS.
        if !self.resize || self.decorated || cfg!(target_os = "macos") {
            return None;
        }
        let window = self.window.as_ref()?;
        if window.is_maximized() || window.fullscreen().is_some() {
            return None;
        }
        #[allow(clippy::cast_possible_truncation)]
        let cursor = Point::new(
            (self.cursor.0 / self.scale_factor) as f32,
            (self.cursor.1 / self.scale_factor) as f32,
        );
        resize_grip(
            self.runtime.viewport(),
            self.runtime.viewport_corner_radii(),
            cursor,
            EDGE_WIDTH,
            None,
        )
    }

    /// Update the system cursor when it crosses a resize grip boundary.
    pub(super) fn update_resize_cursor(&mut self, direction: Option<ResizeDirection>) {
        if self.resize_cursor != direction {
            self.resize_cursor = direction;
            if let Some(window) = &self.window {
                window.set_cursor(direction.map_or(CursorIcon::Default, CursorIcon::from));
            }
        }
    }

    /// Ask the window system to take over an edge drag.
    pub(super) fn try_edge_resize(&self, direction: ResizeDirection) -> bool {
        self.window
            .as_ref()
            .is_some_and(|window| window.drag_resize_window(direction).is_ok())
    }
}

/// Find the edge or corner on the inside of the visible window outline.
pub(super) fn resize_grip(
    size: Size,
    radii: CornerRadii,
    point: Point,
    edge: f32,
    corner: Option<f32>,
) -> Option<ResizeDirection> {
    use ResizeDirection::{East, North, NorthEast, NorthWest, South, SouthEast, SouthWest, West};

    let left = point.x;
    let right = size.width - point.x;
    let top = point.y;
    let bottom = size.height - point.y;
    if [left, right, top, bottom].into_iter().any(|d| d < 0.0) {
        return None;
    }
    let limit = size.width.min(size.height) / 4.0;
    let grip = edge.min(limit);
    let corner = corner.map(|corner| corner.min(limit));
    if grip <= 0.0 {
        return None;
    }

    let mut corner_grip = None;
    for (x, y, radius, direction) in [
        (left, top, radii.top_left, NorthWest),
        (right, top, radii.top_right, NorthEast),
        (right, bottom, radii.bottom_right, SouthEast),
        (left, bottom, radii.bottom_left, SouthWest),
    ] {
        if radius.is_rounded() && x < radius.x && y < radius.y {
            let dx = radius.x - x;
            let dy = radius.y - y;
            if ellipse_value(dx, dy, radius.x, radius.y) > 1.0 {
                return None;
            }
            let inner_x = radius.x - grip;
            let inner_y = radius.y - grip;
            if inner_x <= 0.0 || inner_y <= 0.0 || ellipse_value(dx, dy, inner_x, inner_y) >= 1.0 {
                corner_grip = Some(direction);
            }
        }
        if corner.is_some_and(|corner| x < corner && y < corner)
            || (x < grip && y < grip * 2.0)
            || (y < grip && x < grip * 2.0)
        {
            corner_grip = Some(direction);
        }
    }

    corner_grip.or_else(|| {
        if top < grip {
            Some(North)
        } else if bottom < grip {
            Some(South)
        } else if left < grip {
            Some(West)
        } else if right < grip {
            Some(East)
        } else {
            None
        }
    })
}

/// Evaluate the normalized squared distance from an ellipse center.
fn ellipse_value(x: f32, y: f32, radius_x: f32, radius_y: f32) -> f32 {
    (x / radius_x).powi(2) + (y / radius_y).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizes_on_the_visible_corner_arc_and_not_in_the_cutout() {
        let size = Size::new(120.0, 80.0);
        let radii = CornerRadii::elliptical(24.0, 16.0);
        assert_eq!(
            resize_grip(size, radii, Point::new(2.0, 2.0), EDGE_WIDTH, None),
            None
        );
        assert_eq!(
            resize_grip(size, radii, Point::new(8.0, 5.0), EDGE_WIDTH, None),
            Some(ResizeDirection::NorthWest)
        );
        assert_eq!(
            resize_grip(size, radii, Point::new(17.0, 12.0), EDGE_WIDTH, None),
            None
        );
        assert_eq!(
            resize_grip(size, radii, Point::new(60.0, 2.0), EDGE_WIDTH, None),
            Some(ResizeDirection::North)
        );
    }

    #[test]
    fn square_corners_and_straight_edges_stay_resizable() {
        let size = Size::new(120.0, 80.0);
        let radii = CornerRadii::default();
        for (x, y, direction) in [
            (2.0, 2.0, ResizeDirection::NorthWest),
            (60.0, 2.0, ResizeDirection::North),
            (118.0, 2.0, ResizeDirection::NorthEast),
            (118.0, 40.0, ResizeDirection::East),
            (118.0, 78.0, ResizeDirection::SouthEast),
            (60.0, 78.0, ResizeDirection::South),
            (2.0, 78.0, ResizeDirection::SouthWest),
            (2.0, 40.0, ResizeDirection::West),
        ] {
            assert_eq!(
                resize_grip(size, radii, Point::new(x, y), EDGE_WIDTH, None),
                Some(direction)
            );
        }
        assert_eq!(
            resize_grip(size, radii, Point::new(60.0, 40.0), EDGE_WIDTH, None),
            None
        );
    }
    #[test]
    fn touch_grips_cover_all_edges_and_corners() {
        let size = Size::new(120.0, 80.0);
        let radii = CornerRadii::elliptical(14.0, 14.0);
        for (x, y, direction) in [
            (14.0, 14.0, ResizeDirection::NorthWest),
            (60.0, 8.0, ResizeDirection::North),
            (106.0, 14.0, ResizeDirection::NorthEast),
            (112.0, 40.0, ResizeDirection::East),
            (106.0, 66.0, ResizeDirection::SouthEast),
            (60.0, 72.0, ResizeDirection::South),
            (14.0, 66.0, ResizeDirection::SouthWest),
            (8.0, 40.0, ResizeDirection::West),
        ] {
            assert_eq!(
                resize_grip(size, radii, Point::new(x, y), 10.0, Some(24.0)),
                Some(direction)
            );
        }
        assert_eq!(
            resize_grip(size, radii, Point::new(2.0, 2.0), 10.0, Some(24.0)),
            None
        );
        assert_eq!(
            resize_grip(size, radii, Point::new(60.0, 40.0), 10.0, Some(24.0)),
            None
        );
    }
}
