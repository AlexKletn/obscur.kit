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

| Crate | Depended on by | Verdict |
|---|---|---|
| `obscur-routing` | the facade barrel only | **unwired** — no runtime consumer |
| `obscur-windowing` | the winit and layer adapters + the barrel | **partly wired** — its `layer` module is what `LayerApp` is configured with. The older `Window` / `WindowProperties` still have no consumer: `App` uses winit's own types |
| `obscur-system` | the four platform adapters + the barrel | **host-wired** — `KitBootstrap::system_shell_provider()` hands one over; nothing in the runtime calls it |
| `obscur-testkit` | **nothing at all**, and not re-exported | **unreachable by construction** |

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
| **Indefinite axes and percentages** | percentages should resolve only against a definite size; the constraint model does not carry the distinction everywhere. Related and *fixed*: a percentage block-size resolved against the space a child's *siblings* had left over, compounding down the chain (two `height: 100%` siblings measured 240/0). Flow now attaches the containing block's own definite content size as `BoxConstraints::percent_basis`, separately from the offer. Also *fixed*: a grid item's children saw the grid **container's** content box rather than the item's grid area (a `width: 100%` item was its 211px track while its child resolved `50%` against 432), and grid now states each item's area as `LayoutOutput::child_percent_basis`. Also *fixed*, the same defect in flex: an item's cross size was measured with the main axis pinned to its used size and no basis, so a `width: 50%` item re-read that pinned number as its containing block and laid its own content out at half again (a wrapping inner row reported two lines, 120, where its arranged self has one, 60). The cross measure now states the flex container's content box, which is the number the item's main size was already resolved against. Still open in the same place: only the grid item's **inline** axis is stated, so a percentage *height* on a grid item still resolves against the grid container's content height rather than its row; and a percentage against a genuinely indefinite axis still folds against the offer instead of behaving as `auto` (`specs/w3c/css-sizing-3.txt:554`) — the intrinsic-sizing probes in flex and grid are where that shows |
| **No real min/max-content pair** | intrinsic sizing computes one number where the algorithm wants two |

## What this page is not

It is not a roadmap and not an apology. Most of these are deliberate: failing
closed on `@container` is better than matching wrongly, and a CPU renderer that
approximates GPU effects by policy is better than one that pretends. The list
exists so that reading a type's presence in the re-export barrel does not mislead
you into thinking something runs.

If you need one of these wired, the gap is usually small and specific — most
entries above are "no caller", not "no implementation".
