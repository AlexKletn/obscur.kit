The hexagonal port for platform window management (spec §6.1-6.6).

This module defines [`WindowBackend`], the single trait through which the
pure windowing domain drives the outside platform: lifecycle (§6.1), dynamic
property updates (§6.2), activation and element focus (§6.3), custom chrome
regions (§6.4), fullscreen and presentation (§6.5), and display enumeration
(§6.6), plus capability introspection and event draining. Interface adapters
(a `winit` backend, a WASM backend, a headless test backend) implement it;
the domain never talks to a platform directly.

The guiding contract, spec-wide: every operation returns a **typed result**
and reports **honest capability levels**. A backend never panics on an
unsupported operation — it returns [`WindowOperationResult::Unsupported`],
[`SupportLevel::Unavailable`], or [`ActivationResult::Unsupported`] as
appropriate. The colocated [`NullWindowBackend`] is a reference no-op
implementation that documents this honest-capability contract concretely and
makes the port testable.
