# 12 — What is not wired

A type existing is not evidence that anything calls it, and a doc comment
asserting the wiring is not evidence either. This page is the list of things that
compile, pass their tests, and have **no production caller** — plus the things
that are real but stop at a named boundary.

**Method.** For each entry: grep the symbol across `crates/`, excluding its own
crate and excluding `tests/`. A crate-level entry additionally checks which
`Cargo.toml` files depend on it. Verified at the commit that added this page;
it decays as soon as someone adds the caller.

---

## Unwired by construction: the re-export barrel

`obscur-kit/src/lib.rs` is `pub use X as y;` and nothing else. A crate whose only
path to the outside is that file is exported, compiles, and is called by nothing.

The table below is what the check found. Three rows are still that shape; the
fourth, `obscur-testkit`, used to be the worst of them and no longer is — it is
kept here so the correction is visible next to the claim it replaces.

| Crate | Depended on by | Verdict |
|---|---|---|
| `obscur-routing` | the facade barrel only | **unwired** — no runtime consumer |
| `obscur-windowing` | the winit and layer adapters + the barrel | **partly wired** — its `layer` module is what `LayerApp` is configured with. The older `Window` / `WindowProperties` still have no consumer: `App` uses winit's own types |
| `obscur-system` | the four platform adapters + the barrel | **host-wired** — `KitBootstrap::system_shell_provider()` hands one over; nothing in the runtime calls it |
| `obscur-testkit` | `obscur-runtime`'s test targets, as a dev-dependency | **wired, for tests only.** This entry used to read *"unreachable by construction — depended on by nothing, and not re-exported"*, and that was true of a 49-line skeleton of eleven unit structs. It is now the §2.4 harness — 36 files, ~1,900 lines, 131 public items across `fixture/`, `selector/`, `assertion/`, `mock/` and `host/` — and its **first and so far only consumer is `obscur-runtime`'s own suite**: the 26 files under `crates/application/obscur-runtime/tests/` (233 tests) that the runtime-decomposition campaign moved out of `ui_runtime.rs`, plus a few unit tests still in `src/` that borrow the parts they can reach. It is still deliberately **not** re-exported from the facade: nothing a host ships depends on it, and `PluginTestHost` is still a stub because nothing drives plugin activation through a harness yet. Note the limit the crate boundary imposes: `UiRuntime` is built twice in `obscur-runtime`'s own test target (the library `obscur-testkit` links, and the library under `--test`), so the fixtures that *name* a `UiRuntime` are reachable from `tests/` only, and the unit tests in `src/` can borrow just the parts whose types come from elsewhere — views, stylesheets, pointer events, display lists, key/pointer events, resource providers, `BlockingExecutor` |

## Zero-caller types

| Symbol | Notes |
|---|---|
| `IntrinsicMeasure` | the layout port leaves are documented as routing to. They do not |
| `StyleCache`, `LayoutCache` | dead; both *could* genuinely be wired |
| Vello `GlyphCache` / `PathCache` / `TextureCache` | dead scaffolding — Vello's own atlases make them redundant. To delete, not to bound |
| `GamepadEvent`, `GamepadButton`, `GamepadAxis`, `GamepadId` | modelled; nothing produces one |
| `HapticRequest`, `HapticPattern` | modelled; nothing consumes one |
| `TabOrder`, `FocusTree`, `SpatialNavigation`, `NavigationDirection` | focus traversal; **Tab does not move focus** |
| `GestureArena`, `GestureContender` | the runtime resolves gestures through its own state instead |
| `InjectionSink` | no shipped host installs one, so DevTools injection is always refused |
| `ReplaySession` and the record/replay tier | only a spec test drives it |

## Real, but bounded

| Area | Where it stops |
|---|---|
| **`@container`** | parses, then lowers to `RuleCondition::Never`. Fails closed — the rule never matches. Container query *units* resolve against the viewport |
| **`::before` / `::after`** | parse, then reported unsupported. Generate nothing, fail closed |
| **`ch` units** | layout measures them; paint-side properties read absolute px, so a `ch` that reaches one contributes `0` |
| **Layer-shell keyboard** | `set_keyboard_interactivity` is sent, but no keys are delivered: there is no keysym→`PhysicalKey` table, and an approximate one would put *wrong* keys into an application. `Exclusive` is therefore worse than useless today — it takes the desktop's focus and forwards nothing |
| **Layer-shell popups** | `get_popup` is not exposed; a panel cannot open a menu. Needs xdg-popup alongside |
| **Layer-shell v5** | `set_exclusive_edge` is unreachable — SCTK 0.19 binds versions 1–4 |
| **Font enumeration** | Linux only. macOS and Windows have providers with no enumeration |
| **Font fallback** | per *run*, not per character; `.ttf`/`.otf` only, no `.ttc` collections |
| **Localization** | CLDR plural categories are real, with one production consumer (the shaper's language hint). Dates, numbers, currencies and the RTL frame path are open |
| **Accessibility** | outbound only, behind the non-default `accesskit` feature. Inbound (an AT driving the app) is not built |
| **§6.7 action model** | seven surfaces named; two exist (shortcuts, `Menu`). Custom context menus are required and unbuilt |
| **Components** | authorable now: `<Card/>` binds to the id `#[derive(Contract)]` computes for that name, and `register_component` mounts a factory under it. What stops is the *checking* — an unregistered or misspelled tag mounts nothing with no diagnostic, and the `okui!` macro cannot ever check it (the registry is a runtime value of trait objects; the macro runs while that crate compiles) |
| **`<VirtualFor>`** | compiles on both front ends and emits the §4.9 marker the runtime windows. The template carries **no item binding**, so every realized row is identical; per-item rows still need `register_virtual_list` |
| **Slots** | declared, enforced and projected — but an outlet's name is checked against nothing, so a component rendering `<slot name="hedaer"/>` receives nothing. No shipped component declares a slot: no shipped widget is a `Component` |
| **Telemetry** | `PerformanceSpan` carries no duration; the DevTools protocol has no timing field. `OBSCUR_FRAME_STATS` is the honest profiler |
| **Diagnostics** | several producers construct a `Diagnostic` and drop it into the null sink. Reachable, not reached |
| **Invalidation** | per-node on the style phase; a boolean per phase everywhere else. The phases take no dirty set |
| **Docking** | panels are described and laid out; re-docking by drag is not built |
| **Plugin tier** | an honest, well-labelled, entirely unbuilt skeleton. The `sandbox` module *describes* a sandbox; it does not enforce one, and the crate docs say so |

## Known defects

| | |
|---|---|
| **Flex clamps its own border box** | a flex container clamps rather than overflowing, so `width: 100%` + padding gives a different used width under flex than under block. The chain `[1000, 1020, 960]` is internally inconsistent. Related and *fixed*: an `auto` basis now resolves to max-content, so a plain item no longer eats the free space its growing siblings asked for |
| **Indefinite axes and percentages** | percentages should resolve only against a definite size; the constraint model does not carry the distinction everywhere. Related and *fixed*: a percentage block-size resolved against the space a child's *siblings* had left over, compounding down the chain (two `height: 100%` siblings measured 240/0). Flow now attaches the containing block's own definite content size as `BoxConstraints::percent_basis`, separately from the offer. Also *fixed*: a grid item's children saw the grid **container's** content box rather than the item's grid area (a `width: 100%` item was its 211px track while its child resolved `50%` against 432), and grid now states each item's area as `LayoutOutput::child_percent_basis`. Also *fixed*, the same defect in flex: an item's cross size was measured with the main axis pinned to its used size and no basis, so a `width: 50%` item re-read that pinned number as its containing block and laid its own content out at half again (a wrapping inner row reported two lines, 120, where its arranged self has one, 60). The cross measure now states the flex container's content box, which is the number the item's main size was already resolved against. **Both of the remainders this row used to carry are now closed too.** (i) The grid states each item's area on **both** axes, not just the inline one — `specs/w3c/css-grid-2.txt:3527` is explicit that by the time items are laid out "each grid area's width and height are considered definite for this purpose", which answers by phase order the worry (a content-sized row's extent is the item's own height) that had kept the block axis unstated; a `height: 100%` item's child read 96, half the container's content height, where it should read half the item's 100px row (`tests/percentage_in_grid_item.rs`). (ii) A percentage against a genuinely indefinite axis now **behaves as auto** (`specs/w3c/css-sizing-3.txt:554`) instead of folding against the offer: flow states an indefinite block size as `f32::INFINITY`, which `Dimension::resolve` already answers `None` to, so the value arrives on the `auto` path every caller already had. Two `height: 50%` siblings under an `auto`-height parent measured `(225, 112.5)` — half the offer, then half of what that left — and now measure the `auto` answer, verified equal to the `auto` spelling rather than to a constant (`tests/percentage_against_indefinite.rs`). The rule now reaches the three intrinsic-sizing probes this row used to name: flex's `probe_main` states an infinite main basis (its comment used to read "this engine has no behave-as-auto"; it does now), and grid's step-2 item measure states `(area, ∞)` because the rows are sized *from* that measure and the block axis is genuinely indefinite at that moment. What remains is narrower than the heading: `min-content`/`max-content` as a *cross* size still reads as "not a length" and so is treated as `auto` where css-flexbox scopes stretch to "computed cross size is auto" — a separate sentence, separately open |
| **No real min/max-content pair** | intrinsic sizing computes one number where the algorithm wants two |

## What this page is not

It is not a roadmap and not an apology. Most of these are deliberate: failing
closed on `@container` is better than matching wrongly, and a CPU renderer that
approximates GPU effects by policy is better than one that pretends. The list
exists so that reading a type's presence in the re-export barrel does not mislead
you into thinking something runs.

If you need one of these wired, the gap is usually small and specific — most
entries above are "no caller", not "no implementation".
