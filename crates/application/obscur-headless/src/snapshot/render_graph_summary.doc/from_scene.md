Derives the backend-neutral render graph for an ordinary headless frame:
a single `"scene"` pass covering `scene`'s display list, rendering into a
`target`-sized area.

The pass counts come straight from the scene's
[`SceneStatistics`](obscur_scene::scene::SceneStatistics), so capturing
the same scene twice yields an equal summary (spec §7.14).
