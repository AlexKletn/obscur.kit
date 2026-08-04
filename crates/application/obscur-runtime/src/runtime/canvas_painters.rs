#![doc = include_str!("canvas_painters.doc/_module.md")]

use std::collections::HashMap;

use obscur_identity::{InternedSymbol, NodeId};
use obscur_scene::command::{CanvasRegion, DisplayListBuilder, PaintCommand};
use obscur_tree::tree::RetainedTree;
use obscur_view::view::{CanvasPaint, canvas_painter};

use crate::orchestration::LayoutBoxes;

#[doc = include_str!("canvas_painters.doc/CanvasPainter.md")]
pub type CanvasPainter = Box<dyn FnMut(&mut CanvasRegion<'_>, &CanvasPaint<'_>) -> bool>;

#[doc = include_str!("canvas_painters.doc/CanvasPainters.md")]
#[derive(Default)]
pub(crate) struct CanvasPainters {
    #[doc = include_str!("canvas_painters.doc/painters_field.md")]
    painters: HashMap<InternedSymbol, CanvasPainter>,
    #[doc = include_str!("canvas_painters.doc/unbalanced.md")]
    unbalanced: Vec<InternedSymbol>,
}

impl CanvasPainters {
    #[doc = include_str!("canvas_painters.doc/new.md")]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("canvas_painters.doc/register.md")]
    pub(crate) fn register(&mut self, id: InternedSymbol, painter: CanvasPainter) {
        self.painters.insert(id, painter);
    }

    #[doc = include_str!("canvas_painters.doc/take_unbalanced.md")]
    pub(crate) fn take_unbalanced(&mut self) -> Vec<InternedSymbol> {
        std::mem::take(&mut self.unbalanced)
    }

    #[doc = include_str!("canvas_painters.doc/paint.md")]
    pub(crate) fn paint(
        &mut self,
        tree: &RetainedTree,
        boxes: &LayoutBoxes,
        elapsed: f32,
    ) -> (HashMap<NodeId, Vec<PaintCommand>>, bool) {
        let mut painted = HashMap::new();
        let mut animating = false;
        // An application that registered no painter pays one branch. The walk
        // below is over every live node, and a canvas is rare — most trees hold
        // none at all.
        if self.painters.is_empty() {
            return (painted, animating);
        }
        for (node, retained) in tree.iter() {
            let Some(id) = canvas_painter(retained.props()) else {
                continue;
            };
            let Some(painter) = self.painters.get_mut(&id) else {
                // A canvas naming a painter nobody registered draws nothing.
                // That is the same shape as an `on:click` bound to a command
                // with no handler, and it is not this type's business to report.
                continue;
            };
            let Some(bounds) = boxes.get(&node).copied() else {
                // Laid out nowhere — a display:none branch, or a frame that
                // composed before layout ran. Nothing to paint into.
                continue;
            };
            let mut builder = DisplayListBuilder::new();
            let mut region = CanvasRegion::open(
                &mut builder,
                obscur_geometry::Rect::new(bounds.origin(), bounds.size()),
            );
            animating |= painter(&mut region, &CanvasPaint::new(retained.props(), elapsed));
            if !region.close() {
                self.unbalanced.push(id);
            }
            let commands = builder.build().into_commands();
            if drew_anything(&commands) {
                painted.insert(node, commands);
            }
        }
        (painted, animating)
    }
}

#[doc = include_str!("canvas_painters.doc/drew_anything.md")]
fn drew_anything(commands: &[PaintCommand]) -> bool {
    // A region is never *empty*: opening one emits a transform and a clip, and
    // closing it emits their pops. So "did this painter draw" is not a length
    // test — it is whether any command is something other than a bracket. Asking
    // by count would tie this to how many brackets a region happens to open.
    commands.iter().any(|command| {
        !matches!(
            command,
            PaintCommand::PushTransform(_)
                | PaintCommand::PopTransform
                | PaintCommand::PushClip(_)
                | PaintCommand::PopClip
                | PaintCommand::PushLayer(_)
                | PaintCommand::PopLayer
        )
    })
}

impl std::fmt::Debug for CanvasPainters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The painters are closures and cannot be shown; the count is what a
        // reader of a runtime dump actually wants.
        f.debug_struct("CanvasPainters")
            .field("painters", &self.painters.len())
            .field("unbalanced", &self.unbalanced)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::CanvasPainters;
    use obscur_geometry::{Point, Rect, Size};
    use obscur_identity::{InternedSymbol, NodeId};
    use obscur_scene::color::Color;
    use obscur_scene::command::PaintCommand;
    use obscur_scene::paint::Brush;
    use obscur_tree::node::NodeKind;
    use obscur_tree::tree::RetainedTree;

    use crate::orchestration::LayoutBoxes;

    const CHART: InternedSymbol = InternedSymbol::from_raw(0x0F01);

    /// A tree holding one `<canvas>` naming `CHART`, plus its laid-out box.
    fn one_canvas() -> (RetainedTree, LayoutBoxes, NodeId) {
        let mut tree = RetainedTree::new();
        let node = tree.alloc(NodeKind::Element {
            component: obscur_identity::ComponentId::from_raw(0x00C0_0005),
        });
        tree.set_root(node);
        tree.get_mut(node)
            .expect("live")
            .set_props(vec![obscur_view::view::canvas_painter_prop(CHART)].into_boxed_slice());
        let mut boxes = LayoutBoxes::new();
        boxes.insert(
            node,
            obscur_layout::model::LayoutBox::new(Rect::new(
                Point::new(10.0, 20.0),
                Size::new(100.0, 50.0),
            )),
        );
        (tree, boxes, node)
    }

    fn red() -> Brush {
        Brush::solid(Color::rgba(1.0, 0.0, 0.0, 1.0))
    }

    #[test]
    fn a_registered_painter_paints_into_its_own_node() {
        let (tree, boxes, node) = one_canvas();
        let mut painters = CanvasPainters::new();
        painters.register(
            CHART,
            Box::new(|region, _paint| {
                let size = region.size();
                region
                    .painter()
                    .fill_rect(Rect::new(Point::new(0.0, 0.0), size), red());
                false
            }),
        );

        let painted = painters.paint(&tree, &boxes, 0.0).0;
        let commands = painted.get(&node).expect("the canvas painted");
        assert!(
            commands
                .iter()
                .any(|c| matches!(c, PaintCommand::DrawPath { .. })),
            "the fill reached the list: {commands:?}"
        );
    }

    #[test]
    fn nothing_registered_costs_one_branch_and_paints_nothing() {
        let (tree, boxes, _node) = one_canvas();
        let mut painters = CanvasPainters::new();
        assert!(painters.paint(&tree, &boxes, 0.0).0.is_empty());
    }

    #[test]
    fn a_canvas_naming_an_unregistered_painter_draws_nothing_and_does_not_panic() {
        let (tree, boxes, _node) = one_canvas();
        let mut painters = CanvasPainters::new();
        painters.register(InternedSymbol::from_raw(0x0F99), Box::new(|_, _| false));
        assert!(painters.paint(&tree, &boxes, 0.0).0.is_empty());
    }

    #[test]
    fn a_painter_that_draws_nothing_contributes_no_entry() {
        // A region is never empty — it always brackets itself — so "drew
        // nothing" has to be asked about the CONTENT. A canvas whose painter
        // drew nothing must not cost a map slot and a splice of four no-ops
        // every frame.
        let (tree, boxes, _node) = one_canvas();
        let mut painters = CanvasPainters::new();
        painters.register(CHART, Box::new(|_, _| false));
        assert!(painters.paint(&tree, &boxes, 0.0).0.is_empty());
    }

    #[test]
    fn an_unbalanced_painter_is_recorded_by_name_and_reported_once() {
        let (tree, boxes, _node) = one_canvas();
        let mut painters = CanvasPainters::new();
        painters.register(
            CHART,
            Box::new(|region, _paint| {
                region
                    .painter()
                    .push_clip(obscur_scene::clip::Clip::rect(Rect::new(
                        Point::new(0.0, 0.0),
                        Size::new(1.0, 1.0),
                    )));
                false
            }),
        );

        let _ = painters.paint(&tree, &boxes, 0.0);
        assert_eq!(
            painters.take_unbalanced(),
            vec![CHART],
            "the painter that left a bracket open is named"
        );
        assert!(
            painters.take_unbalanced().is_empty(),
            "and taken, so a host reports it once rather than every frame"
        );
    }

    #[test]
    fn a_canvas_with_no_laid_out_box_is_skipped() {
        let (tree, _boxes, _node) = one_canvas();
        let mut painters = CanvasPainters::new();
        painters.register(
            CHART,
            Box::new(|region, _paint| {
                region
                    .painter()
                    .fill_rect(Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0)), red());
                false
            }),
        );
        assert!(
            painters.paint(&tree, &LayoutBoxes::new(), 0.0).0.is_empty(),
            "a display:none branch has nowhere to paint into"
        );
    }
}
