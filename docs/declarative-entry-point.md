# A declarative entry point

**Status:** design note, nothing built. Written 2026-07-17 at `cb7a704`, after the spec-conformance campaign, as the brief for the DX work that follows it.

## The target

> `create_app(config)` + `.okui` screens and components should just work. A developer must not have to think about which platform this runs on.

That is the requirement in the owner's words. This note records the measured gap, the naming problem, the principle that governs the shape, and the forks that are genuinely open.

## The measured gap

There is no such entry point. To put a counter on screen today a host writes, by hand:

| example | lines |
|---|---|
| `showcase_window.rs` | 938 |
| `windowed_counter.rs` | 891 |
| `wireframe_live_window.rs` | 730 |
| `uikit_window.rs` | 688 |
| `controls_window.rs` | 630 |
| `capabilities_window.rs` | 403 |

To reach a window a developer must personally know `winit::EventLoop`, `VelloDevice`, `VelloSurface`, `window.set_ime_allowed`, and must register fonts under ids that match whatever the runtime's resolver assigned. Every one of those is a platform detail the target says they must not think about.

**27 examples hand-wire the same text stack.** The facade already knows how — `KitBootstrap` builds `HarfrustShaper`, `FontRegistry` and `UnicodeTextItemizer` — and until round five it had no installer at all, so every example retyped the answer the kit was holding.

Six installers exist today and a host must call each one it wants: `install_default_stylesheet`, `install_image_resources`, `install_vector_resources`, `install_accessibility`, `install_blocking_executor`, `install_text_stack`.

## This is not only ergonomics

Boilerplate and the "built but not wired" defect are the same finding seen from two sides. The conformance audit kept asking *who calls this?* and kept getting "nobody" — and the reason was usually that calling it was inconvenient:

- Six windowed examples reach around `WindowBackend` to winit directly. Round six's verdict was that the port is **not** misshapen and deferred creation is **not** the blocker: it composes fine and **nobody had tried**.
- `obscur-windowing` is ~3,100 lines with zero production consumers.
- `AccessKitBridge` was never constructed outside its own crate, so a correct semantic tree was published to nobody.
- Until round nine, `Spawner`/`Timer`/`CancellationToken` had zero runtime consumers and `default_task_spawner()` returned an object nothing could accept.

So a declarative entry point is not sugar over finished parts. **It is the missing consumer that would have forced those ports to be real** — and would have found their defects years earlier, the way wiring the a11y bridge immediately exposed that `SemanticSnapshot` was dropping the tree's parent/child links and every adapter was receiving a flat tree.

## The name

`create_app` is the wrong name, and the reason is the spec's own vocabulary.

§5.4 *Overlay host mode* uses **host** for the **external engine** that owns the window, event loop, render graph and lifecycle. The kit is the guest: it receives viewport, input, clock, render target and frame-scheduling hooks, and returns render commands. §1.4 requires the same — "embedded mode inside an existing host window, surface or event loop".

§3.2 is explicit that the kit is not the application: it borrows the intent vocabulary and states it "does not recreate a full application Intent Machine, sync engine or global overlay model". §7.2 does the same for translation: "The toolkit is not a full application translation-management platform."

So the UI is a *part* of an app, not the app — and in this spec's vocabulary **host means the application**, which `create_app` would invert. `HeadlessHost` already follows the correct convention: it is a driver standing outside the runtime and feeding it.

The thing to build is therefore **a ready-made host the kit offers** for the case where you do not want to write your own — a windowed sibling to `HeadlessHost`, not an "app".

## The principle, and its boundary

Encapsulation hides **how**, not **what happened**.

- *Hide entirely:* winit on Linux, `web_sys` on wasm, which shaper, which renderer. A developer neither knows nor cares.
- *Never hide:* the compositor denied focus; this platform cannot do exclusive fullscreen; no font was supplied so no text will paint.

§6 already does this correctly and is the model to copy: one interface, and `native()` / `emulated()` / `conditional()` / `Unsupported` tell you what actually happened. You may ignore the result; if you need it, you ask **one way** rather than through `#[cfg(target_os)]`.

The failure mode to avoid has a precedent in this repo. §6-B was an interface that hid the platform and **lied uniformly**: `native()` returned for owner relations, size constraints and transparency that were silently dropped. Simple to call, no platform knowledge required, and the window was not what you asked for. A convenient entry point that reports success for work it did not do would be that defect on the most visible surface in the kit.

### Where defaults win, and where they must not

The campaign's other rule is honesty, and several installers are opt-in **deliberately** — round nine kept `install_blocking_executor` opt-in citing §3.3's "blocking work is opt-in and capability-dependent". Minimum boilerplate pulls toward defaults-on; honesty pulls toward explicit. The line that resolves it:

- **Where the spec promises it works out of the box, the default wins.** §5.18 requires "usable rendering out of the box", and the production default was a *mock* shaper — that was boilerplate *and* a false claim at once.
- **Where the spec makes something capability-dependent or host-supplied, an explicit call is correct** and is not boilerplate. Fonts in WASM are host-supplied by decision (see §5.18's recorded deviation); threads are opt-in per §3.3.

## Open forks

Not decisions to be made alone:

1. **A type or a function?** A host object that lives and can be extended, versus a call that returns something you own. The spec is indifferent; the DX difference is large.
2. **What is in the config?** Every knob added is a thing a developer must think about — the opposite of the target. The default set has to be defensible without reference to a platform.
3. **How does a headless test reach the same surface?** §1.4 requires headless mode with no window and no GPU, and §7.3/§7.4 require determinism. Whatever this is, it must not become a second path that only windowed apps take, or the harness and the product drift.
4. **What does it do when the platform cannot?** No GPU, no fonts in a WASM host, no compositor. Falling back silently would be §6-B again; refusing to start may be worse. The answer is probably a typed result on the same shape §6 already uses.

## How to know it worked

The measurement is already in place: **`windowed_counter.rs` is 891 lines.** A declarative entry point that does not move that number substantially has not done the job. And the check that transfers from the conformance campaign is unchanged — *who calls it?* If the only caller is a test, it is either unwired or so inconvenient that nobody wired it. The first is a conformance defect; the second is a DX one.
