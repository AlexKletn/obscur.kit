End-to-end verification that an **SVG `src` paints as vector, and follows the
computed `color`** through the *real* live pipeline (spec §5.17).

§5.17 is two sentences and this file checks both, against a real file read off
disk by the sandbox-rooted `FsResourceLoader`:

- *"SVG remains vector through the scene pipeline where possible"* — the icon
  reaches the scene as `DrawPath` outlines and never as a rasterized
  `DrawImage`, and the CPU backend really draws it;
- *"`currentColor` … resolves from computed `color`, allowing theme/state
  changes **without reparsing**"* — the same loaded document, composed under a
  different computed `color`, paints a different colour. The runtime parses a
  source once and caches it colour-independently, so the second frame does no
  parsing at all.

This closes the audit's §5 finding that SVG was decoded and validated but never
painted by anything.
