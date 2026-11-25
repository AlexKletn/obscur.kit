One effect a [`DisplayList`](obscur_scene::command::DisplayList) can demand of
a [`Renderer`](crate::renderer::Renderer) (spec §5.2).

Spec §5.2 requires that "unsupported effects and fallbacks are explicit
diagnostics". An explicit diagnostic needs a **closed** vocabulary: a free
string lets a backend answer `Emulated` for a name it has never heard of, which
is how the CPU backend came to claim it emulated `blur` while dropping the blur
radius on the floor. Every variant here is something the display list actually
carries, so a backend's [`supports_effect`](crate::renderer::Renderer::supports_effect)
table is an exhaustive match the compiler checks, and
[`scene_effects`](crate::renderer::scene_effects) can name exactly what a given
scene needs.

The vocabulary covers *effects* — the marks and filters a backend either
reproduces or does not. It deliberately does **not** cover every compositing
parameter a [`SceneLayer`](obscur_scene::layer::SceneLayer) carries: group
opacity, the Porter-Duff operator, and a layer's own transform have no variant
here and are therefore not diagnosable through this port.
