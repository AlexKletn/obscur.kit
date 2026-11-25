The **mutable** Scene/Paint model: a root [`DisplayList`] paired with the
[`SceneGeneration`] stamping its content (spec §1.6, §5.2).

The runtime owns a `Scene` and mutates it while building a frame — for
example by replacing the root with [`Scene::set_root`], which advances the
generation to record that the content changed. When the frame is ready, the
runtime publishes an immutable [`SceneSnapshot`] via [`Scene::snapshot`] and
hands it to a renderer. The renderer only ever sees the snapshot, never this
mutable scene.
