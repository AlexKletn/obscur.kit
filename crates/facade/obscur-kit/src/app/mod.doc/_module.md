[`App`]: the one-call **standalone windowed** entry point (spec §1.4).

This is the §1.4 "standalone mode with kit-created windows" entry — the *one*
§1.4 mode in which the kit may own an OS window and drive the event loop
itself. In every other mode the kit is a **guest**: embedded in a host's window,
composited as an overlay, or run headless for offscreen rendering and tests —
none of which this API serves. A guest host wires the pieces by hand through
[`KitBootstrap`](crate::bootstrap::KitBootstrap) and
[`CssPipeline`](crate::bootstrap::CssPipeline); this type is the standalone
shortcut that composes those same facilities into a booted, painting, input-fed
window so an application does not re-hand-wire the ~700 lines the windowed
examples spell out. It is deliberately gated on the full windowed feature set
(`window-winit` + `render-vello`) and is absent from the headless `default`
build.

```no_run
use obscur_kit::App;
# use obscur_kit::view::view::View;
# fn build() -> View { View::default() }
App::new("Hello")
    .with_size(480.0, 240.0)
    .stylesheet(".root { background-color: #1e2636; }")
    .view(build)
    .run()
    .unwrap();
```

# Reaching the runtime from an application

Three of the runtime's capabilities are *positional* or *live*, so they cannot be
configured by a plain builder call: a handler at one node (§2.6's links 1–3) needs
a `NodeId` that only exists after a frame, and replacing the stylesheets (§4.12)
happens while the loop runs. An `App` reaches them by **name** and by **queue**:

```no_run
use obscur_kit::App;
use obscur_kit::headless::input::SelectorTarget;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::interaction::event::UiEvent;
# use obscur_kit::view::view::View;
const PICK: InternedSymbol = InternedSymbol::named("pick");

let app = App::new("Editor").screen(r#"<div test-id="row" on:click=pick/>"#);
let control = app.control();
app
    // §2.3's payload: which node was hit, where, with what device.
    .on_command(PICK, |e: &mut UiEvent| { let _ = e.target(); })
    // §2.6 link 1: the same symbol, different logic, at one element.
    .on_command_at(SelectorTarget::test_id("row"), PICK, || {})
    // §4.12: the authored set, replaced while running.
    .on_command(InternedSymbol::named("dark"), move || {
        control.set_stylesheet(":root { background-color: #101010 }");
    })
    .run()
    .unwrap();
```

A handler never receives the runtime: it runs *inside* §2.3's propagation, so a
request is recorded and applied at the frame boundary. See
[`AppControl`](crate::control::AppControl) for that contract.

What it turns on for you, honestly (§3.3, §5.18): the real text-shaping stack
over the system fonts (bundled last-resort fallback appended), the UA default
stylesheet plus your authored CSS through the motion-aware
[`CssPipeline`](crate::bootstrap::CssPipeline), filesystem image loading/decode,
the threaded blocking executor (correct here because a standalone window is not
a headless host), IME text input, and HiDPI. See
[`KitBootstrap`](crate::bootstrap::KitBootstrap) for the manual equivalents.
