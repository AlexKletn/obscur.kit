A `.okui` screen that reflects changed state — the case that could not exist.

§2.9 defines `.okui` as a front end "compiled into the same UI IR as the Rust
DSL", a peer of it. The run-time route was not a peer: `App::screen` compiled the
source in `run` and handed the runtime `move || view.clone()`, one pre-built
`View` for the application's whole life ("static v1 materialization"), so **no**
state change could reach the screen. A standalone `.okui` file and reactivity were
mutually exclusive.

What changed is what is kept: the compiled **IR**, not the lowered `View`. Each
frame lowers it again — nothing re-parses, nothing re-compiles — and the lowering
resolves `{bindings}` against a [`ScreenData`] table the application writes. The
last case measures what that costs.

Structure is still fixed: `<If>`/`<For>` are statically materialised on this
route, and a `{binding}` inside a `<VirtualFor>` keeps its placeholder. Shape that
depends on state belongs in the build-time route (a `build.rs` emitting through
`CodegenRoot::facade`, or the inline `okui!` macro), where the markup becomes
ordinary Rust — and where §2.9's compile-time contract validation happens.
