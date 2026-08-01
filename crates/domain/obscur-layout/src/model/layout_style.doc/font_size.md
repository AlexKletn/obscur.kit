The node's font size in logical pixels, used to shape and wrap a text
leaf during the measure/arrange pass (spec §5.10).

Only meaningful for a text node: the coordinator populates it from the
node's computed `font-size` so the constraint-aware measure can shape and
wrap the run to its available width. It defaults to
[`DEFAULT_FONT_SIZE`](Self::DEFAULT_FONT_SIZE) and is ignored by every
box (non-text) algorithm.
