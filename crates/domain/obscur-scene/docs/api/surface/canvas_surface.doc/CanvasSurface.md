A descriptor for a built-in Canvas surface (Obscur Kit 2 specs §5.3).

A Canvas is a custom rendering surface that draws via the common Paint model
and participates in the normal scene: layout, clipping, transforms, hit
testing, input, accessibility, and composition. This type is a pure data
descriptor — Phase 1 models only the descriptor; the actual drawing and
compositing behaviour lands in a later phase.

The `content_version` is a monotonically increasing counter the compositor
can use to detect when the Canvas has redrawn its contents (so cached output
can be invalidated). It is bumped whenever the Canvas content changes.
