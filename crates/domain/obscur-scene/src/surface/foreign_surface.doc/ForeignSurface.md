A descriptor for externally-produced content integrated into the scene
(Obscur Kit 2 specs §5.5).

A foreign surface hosts content produced outside the Paint model — video,
web views, cameras, native controls, or external GPU surfaces. It is either:

- **texture-backed** (`texture_backed == true`): the content is composited as
  a texture and therefore integrates seamlessly, supporting the full
  transform / clip / opacity / blur pipeline; or
- a **native child surface/window** (`texture_backed == false`): the content
  is a platform-managed surface with explicit restrictions.

The `limits` field records, explicitly, which scene operations the surface
permits. Per §5.5 these capabilities are never silently assumed. This type is
a pure data descriptor — it holds no live GPU or renderer handle.
