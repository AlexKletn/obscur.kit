The stable code of the diagnostic raised when a backend *did* draw an effect,
but with one of its documented approximations standing in for what the scene
asked for (spec §5.2).

The sibling of [`UNSUPPORTED_EFFECT`], and the other half of §5.2's "unsupported
effects **and fallbacks** are explicit diagnostics": an `Unavailable` effect is a
property of the backend and is knowable before drawing, whereas a fallback is a
property of the *frame* — a [`SupportLevel::Limited`](obscur_capability::SupportLevel::Limited) effect draws perfectly
until the one input its restriction names is missing. Reporting it needs the
backend to say that the restriction actually bit, which is why this is emitted
from the render path rather than derived from [`Renderer::supports_effect`](crate::renderer::Renderer::supports_effect).
