A parsed vector document (spec §5.17).

# Vector, not pixels

Spec §5.17: "SVG remains vector through the scene pipeline where possible."
This is that representation — outlines and paints, never a rasterized bitmap.
It composes to
[`DrawPath`](crate::command::PaintCommand::DrawPath) commands, which both
renderers already consume, so an SVG is rasterized once at the same
resolution as everything else the frame draws rather than being sampled from
a pre-rendered image.

# Parsed once, drawn in any colour

A shape's paint is a [`VectorPaint`], so `currentColor` survives parsing
unresolved. That is what lets §5.17's "theme/state changes without reparsing"
hold: the document is colour-independent, and the coordinator substitutes the
painting node's computed `color` per frame.

# Scope

Fills and strokes of solid or `currentColor` paint. SVG gradients, patterns,
masks, filters and text are **not** carried: they would each need a mapping
onto the scene's own model, and inventing one would be worse than reporting
the gap. A document using them keeps its geometry; those paints are dropped.
