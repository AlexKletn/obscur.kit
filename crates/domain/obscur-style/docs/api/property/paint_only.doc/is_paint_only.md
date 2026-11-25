Whether this property is **paint-only**: changing it can never change any
node's box, so a frame in which only paint-only properties changed can
reuse the retained layout (spec §3.4 phase 5, §4.3 — invalidation is
classified by what actually changed, not by the frame's reason).

This is a deliberately **conservative allow-list**: a property is
paint-only only when it provably cannot feed measure/arrange. Everything
else — including any property added later — answers `false` via the
catch-all and is treated as layout-affecting, so the failure mode is a
redundant relayout (slow but correct), never a stale box.

Deliberately **not** paint-only despite looking like it:
- `border-*-style`: `none` forces the used border width to zero, so it
  feeds the box.
- `text-align` / `text-overflow`: both are threaded into the text
  measure pass here, so a change is treated as layout-affecting.
