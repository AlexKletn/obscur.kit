A deterministic mock [`IntrinsicMeasure`] for the Phase-1 headless slice
(spec §5.13).

It lets the layout pipeline run without a real text engine or image decoder:
each node is assigned a fixed *natural content size* up front, and the mock
reports that size directly. This is a **test/dev stand-in only** and will be
replaced by the `obscur-text` adapter (and a replaced-content measurer) in a
later phase.

Nodes registered via [`set`](Self::set) return their stored natural size;
any unregistered node falls back to `default_size`. Because the mock has no
reflow, its [`intrinsic_size`](IntrinsicMeasure::intrinsic_size) reports the
natural size as both min-content and max-content.

The provider is deterministic and side-effect-free, satisfying the
[`IntrinsicMeasure`] contract: measurement reads the map but never mutates it.
