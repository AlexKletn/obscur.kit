#![doc = include_str!("inspect.doc/_module.md")]

use std::fmt;

use obscur_geometry::Rect;
use obscur_identity::{ComponentId, NodeId};
use obscur_scene::color::Color;
use obscur_scene::command::PaintCommand;
use obscur_scene::paint::{Brush, Paint, Stroke};
use obscur_scene::path::Path;

#[doc = include_str!("inspect.doc/HIGHLIGHT_FILL.md")]
const HIGHLIGHT_FILL: Color = Color::rgba(0.25, 0.55, 0.95, 0.22);
#[doc = include_str!("inspect.doc/HIGHLIGHT_LINE.md")]
const HIGHLIGHT_LINE: Color = Color::rgba(0.30, 0.62, 1.0, 0.95);
#[doc = include_str!("inspect.doc/OUTLINE_WIDTH.md")]
const OUTLINE_WIDTH: f32 = 1.0;

#[doc = include_str!("inspect.doc/overlay_commands.md")]
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

#[doc = include_str!("inspect.doc/InspectInfo.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct InspectInfo {
    #[doc = include_str!("inspect.doc/node.md")]
    pub node: NodeId,
    #[doc = include_str!("inspect.doc/component.md")]
    pub component: Option<ComponentId>,
    #[doc = include_str!("inspect.doc/local_name.md")]
    pub local_name: Option<String>,
    #[doc = include_str!("inspect.doc/id.md")]
    pub id: Option<String>,
    #[doc = include_str!("inspect.doc/classes.md")]
    pub classes: Vec<String>,
    #[doc = include_str!("inspect.doc/bounds.md")]
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

#[doc = include_str!("inspect.doc/NodeInspection.md")]
#[derive(Debug, Clone, PartialEq)]
pub struct NodeInspection {
    #[doc = include_str!("inspect.doc/node_2.md")]
    pub node: NodeId,
    #[doc = include_str!("inspect.doc/parent.md")]
    pub parent: Option<NodeId>,
    #[doc = include_str!("inspect.doc/depth.md")]
    pub depth: u32,
    #[doc = include_str!("inspect.doc/kind.md")]
    pub kind: &'static str,
    #[doc = include_str!("inspect.doc/component_2.md")]
    pub component: Option<ComponentId>,
    #[doc = include_str!("inspect.doc/local_name_2.md")]
    pub local_name: Option<String>,
    #[doc = include_str!("inspect.doc/id_2.md")]
    pub id: Option<String>,
    #[doc = include_str!("inspect.doc/classes_2.md")]
    pub classes: Vec<String>,
    #[doc = include_str!("inspect.doc/text.md")]
    pub text: Option<String>,
    #[doc = include_str!("inspect.doc/bounds_2.md")]
    pub bounds: Option<Rect>,
    #[doc = include_str!("inspect.doc/styles.md")]
    pub styles: Vec<(String, String)>,
}

impl fmt::Display for NodeInspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.depth {
            f.write_str("  ")?;
        }
        // Identity: a CSS-style selector, or a text node's quoted content.
        if let Some(text) = &self.text {
            write!(f, "‹text› {text:?}")?;
        } else {
            let mut selector = String::new();
            if let Some(name) = &self.local_name {
                selector.push_str(name);
            } else {
                selector.push_str(self.kind);
            }
            if let Some(id) = &self.id {
                selector.push('#');
                selector.push_str(id);
            }
            for class in &self.classes {
                selector.push('.');
                selector.push_str(class);
            }
            f.write_str(&selector)?;
        }
        if let Some(r) = self.bounds {
            write!(
                f,
                "  @{:.0},{:.0} {:.0}×{:.0}",
                r.origin.x, r.origin.y, r.size.width, r.size.height
            )?;
        } else {
            f.write_str("  @(no box)")?;
        }
        write!(f, "  node #{}", self.node.raw())?;
        for (name, value) in &self.styles {
            write!(f, " {name}={value}")?;
        }
        Ok(())
    }
}

#[doc = include_str!("inspect.doc/TreeInspection.md")]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeInspection {
    #[doc = include_str!("inspect.doc/nodes.md")]
    pub nodes: Vec<NodeInspection>,
}

impl TreeInspection {
    #[doc = include_str!("inspect.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[doc = include_str!("inspect.doc/is_empty.md")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl fmt::Display for TreeInspection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, node) in self.nodes.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{node}")?;
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
