//! In-window inspect overlay: highlight the hovered node's box and describe its
//! identity, so a developer can debug the running UI directly in the window
//! rather than from a separate tool (spec §7.6 — inspect mode uses the same
//! hit-testing engine as the runtime and can highlight bounds).
//!
//! [`overlay_commands`] turns a node's absolute border-box rectangle into the
//! two leaf paint commands the runtime appends over the composed scene when
//! inspect mode is on: a translucent fill and a solid outline. [`InspectInfo`]
//! is the host-facing readout the runtime builds from the hovered node — its
//! [`Display`](std::fmt::Display) line names the element (CSS-style
//! `type#id.class`), its geometry, and its component id.

use std::fmt;

use obscur_geometry::Rect;
use obscur_identity::{ComponentId, NodeId};
use obscur_scene::color::Color;
use obscur_scene::command::PaintCommand;
use obscur_scene::paint::{Brush, Paint, Stroke};
use obscur_scene::path::Path;

/// The translucent fill painted over the highlighted node's border box.
const HIGHLIGHT_FILL: Color = Color::rgba(0.25, 0.55, 0.95, 0.22);
/// The solid outline drawn around the highlighted node's border box.
const HIGHLIGHT_LINE: Color = Color::rgba(0.30, 0.62, 1.0, 0.95);
/// The outline width, in logical pixels.
const OUTLINE_WIDTH: f32 = 1.0;

/// Builds the overlay for a node's absolute border-box `bounds`: a translucent
/// fill under a solid outline. Both are leaf [`PaintCommand::DrawPath`]s, so
/// appending them to a balanced display list keeps it balanced.
#[must_use]
pub fn overlay_commands(bounds: Rect) -> Vec<PaintCommand> {
    vec![
        PaintCommand::DrawPath {
            path: Path::rect(bounds),
            paint: Paint::solid_fill(HIGHLIGHT_FILL),
        },
        PaintCommand::DrawPath {
            path: Path::rect(bounds),
            paint: Paint::stroke(Stroke::new(Brush::solid(HIGHLIGHT_LINE), OUTLINE_WIDTH)),
        },
    ]
}

/// A host-facing readout of the node currently under the pointer (spec §7.6).
///
/// Built by the runtime from the hovered node's retained record and its absolute
/// layout box; its [`Display`](fmt::Display) renders a one-line CSS-style summary
/// a host can print next to the in-window highlight.
#[derive(Debug, Clone, PartialEq)]
pub struct InspectInfo {
    /// The hovered node's stable id.
    pub node: NodeId,
    /// The backing component id, for an element node (`None` for group/text).
    pub component: Option<ComponentId>,
    /// The element's local name (CSS type), if any.
    pub local_name: Option<String>,
    /// The element's `id` attribute, if any.
    pub id: Option<String>,
    /// The element's classes, in order.
    pub classes: Vec<String>,
    /// The absolute border-box rectangle, in logical pixels.
    pub bounds: Rect,
}

impl fmt::Display for InspectInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // CSS-style selector: `type#id.a.b`, falling back to `‹node›` when the
        // element carries no name/id/classes (a group or bare element).
        let mut selector = String::new();
        if let Some(name) = &self.local_name {
            selector.push_str(name);
        }
        if let Some(id) = &self.id {
            selector.push('#');
            selector.push_str(id);
        }
        for class in &self.classes {
            selector.push('.');
            selector.push_str(class);
        }
        if selector.is_empty() {
            selector.push_str("‹node›");
        }

        let r = self.bounds;
        write!(
            f,
            "{selector}  ·  {:.0},{:.0} {:.0}×{:.0}  ·  node #{}",
            r.origin.x,
            r.origin.y,
            r.size.width,
            r.size.height,
            self.node.raw(),
        )?;
        if let Some(component) = self.component {
            write!(f, " · cid {:#x}", component.raw())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    fn rect() -> Rect {
        Rect::new(Point::new(20.0, 20.0), Size::new(960.0, 454.0))
    }

    #[test]
    fn overlay_is_a_fill_under_an_outline() {
        let commands = overlay_commands(rect());
        assert_eq!(commands.len(), 2);
        assert!(matches!(
            &commands[0],
            PaintCommand::DrawPath { paint, .. } if paint.is_fill()
        ));
        assert!(matches!(
            &commands[1],
            PaintCommand::DrawPath { paint, .. } if paint.is_stroke()
        ));
    }

    #[test]
    fn info_renders_a_css_style_selector_with_geometry() {
        let info = InspectInfo {
            node: NodeId::from_raw(3),
            component: Some(ComponentId::from_raw(0x00C0_0001)),
            local_name: Some("div".to_owned()),
            id: Some("app".to_owned()),
            classes: vec!["gallery".to_owned(), "dark".to_owned()],
            bounds: rect(),
        };
        let line = info.to_string();
        assert!(line.contains("div#app.gallery.dark"), "{line}");
        assert!(line.contains("20,20 960×454"), "{line}");
        assert!(line.contains("node #3"), "{line}");
        assert!(line.contains("cid 0xc00001"), "{line}");
    }

    #[test]
    fn nameless_node_falls_back_to_placeholder() {
        let info = InspectInfo {
            node: NodeId::from_raw(1),
            component: None,
            local_name: None,
            id: None,
            classes: vec![],
            bounds: rect(),
        };
        assert!(info.to_string().contains("‹node›"));
    }
}
