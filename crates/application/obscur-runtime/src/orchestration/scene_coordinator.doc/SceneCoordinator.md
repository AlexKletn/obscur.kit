Orchestrates the scene phase: turns the styled + laid-out tree into a
Scene/Paint [`DisplayList`] with a deterministic paint order (spec §5.1,
§3.10, §3.4).

The coordinator is **orchestration only**: it walks the retained tree and
emits paint commands through `obscur-scene`'s [`DisplayListBuilder`], but it
performs no rasterization — turning the display list into pixels is a
renderer concern in a later phase.

# Paint order

The tree is walked **pre-order** (a parent is emitted before its children,
children left-to-right); the walk is **cycle-safe** — a visited-set guard
means a shared or cyclic child link terminates instead of looping — and
**panic-free**: every lookup is fallible and no indexing is performed.

Paint order is then the scene's `z-index` **stacking** order (spec §5.1): each
node's own paint is captured as a unit keyed `(effective z, pre-order index)`
and stable-sorted, so a positioned element with a higher `z-index` paints on
top of a lower one, and a positioned subtree stacks together (see the
[`stacking`](super::stacking) module for the bounded, flat model and its
documented simplifications). When no `z-index` is authored every effective z
is `0` and the order is exactly pre-order (parent, then children) — the
pre-stacking behaviour, byte-for-byte. Whatever wraps a whole subtree — a
rounded clip, a transform, an opacity/blend/filter group layer, a scroll
translation, an outline ring — is recorded as a pair by
`SubtreeBrackets` and emitted once
the walk knows where each subtree ends.

# Phase-1 scope

The coordinator emits **background fills** and **shaped text runs**: a node
whose computed `background-color` is present and not fully transparent fills
its arranged layout rectangle, and a text node with an arranged box emits a
[`draw_text`](DisplayListBuilder::draw_text) run at that box's origin.

Text is **shaped once, here in the pipeline**, through an `obscur-text`
[`TextShaper`] port (a deterministic mock by default): the coordinator turns
the node's string into a [`GlyphRun`](obscur_scene::text::GlyphRun) of
positioned glyphs and carries those glyphs in the
[`DrawText`](obscur_scene::command::PaintCommand::DrawText) command, so every
renderer consumes an identical run and only rasterization differs (spec §5.10,
§5.2). Each run is shaped at the node's computed `font-size` and its glyphs sit
on the font's real baseline, the same size and metrics the layout phase
measured the box with. Because only leaf
[`fill_rect`](DisplayListBuilder::fill_rect) and
[`draw_text`](DisplayListBuilder::draw_text) commands are emitted inside the
balanced subtree brackets, the resulting [`DisplayList`] is always well-formed
and can be consumed by a renderer or `Scene` directly.

The coordinator owns its paragraph engine; construct it with
[`new`](Self::new) or [`Default`]. Because that engine holds `dyn` ports it is
neither `Clone` nor `Debug`, so the coordinator is not `Clone` and carries a
manual [`fmt::Debug`].
