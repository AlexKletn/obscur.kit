#![doc = include_str!("canvas.doc/_module.md")]

use obscur_identity::InternedSymbol;

use crate::runtime::{CanvasPainter, PainterKey};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("canvas.doc/register_canvas.md")]
    pub fn register_canvas(&mut self, id: InternedSymbol, painter: CanvasPainter) {
        self.canvas_painters
            .register(PainterKey::Symbol(id), painter);
    }

    #[doc = include_str!("canvas.doc/register_named_canvas.md")]
    pub fn register_named_canvas(&mut self, name: &str, painter: CanvasPainter) {
        self.canvas_painters
            .register(PainterKey::Name(name.into()), painter);
    }

    #[doc = include_str!("canvas.doc/take_unbalanced_canvases.md")]
    pub fn take_unbalanced_canvases(&mut self) -> Vec<PainterKey> {
        self.canvas_painters.take_unbalanced()
    }
}
