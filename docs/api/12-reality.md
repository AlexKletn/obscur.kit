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
| `obscur-windowing` | the winit adapter + the barrel | **unwired on the app path** — `App` uses winit's own window types, not `Window` / `WindowProperties` |
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
| **Font enumeration** | Linux only. macOS and Windows have providers with no enumeration |
| **Font fallback** | per *run*, not per character; `.ttf`/`.otf` only, no `.ttc` collections |
| **Localization** | CLDR plural categories are real, with one production consumer (the shaper's language hint). Dates, numbers, currencies and the RTL frame path are open |
| **Accessibility** | outbound only, behind the non-default `accesskit` feature. Inbound (an AT driving the app) is not built |
| **§6.7 action model** | seven surfaces named; two exist (shortcuts, `Menu`). Custom context menus are required and unbuilt |
| **Components** | `register_component` is real; **no user component can be authored in either front end**, so the component tier is the built-ins |
| **`<VirtualFor>`** | refused on both front ends. Markup-level windowing is unimplemented; the runtime-registered form is the way |
| **Slots** | declared and enforced on both front ends, projected by the runtime — but with no user components, only the built-ins can have them |
| **Telemetry** | `PerformanceSpan` carries no duration; the DevTools protocol has no timing field. `OBSCUR_FRAME_STATS` is the honest profiler |
| **Diagnostics** | several producers construct a `Diagnostic` and drop it into the null sink. Reachable, not reached |
| **Invalidation** | per-node on the style phase; a boolean per phase everywhere else. The phases take no dirty set |
| **Docking** | panels are described and laid out; re-docking by drag is not built |
| **Plugin tier** | an honest, well-labelled, entirely unbuilt skeleton. The `sandbox` module *describes* a sandbox; it does not enforce one, and the crate docs say so |

## Known defects

| | |
|---|---|
| **Flex clamps its own border box** | a flex container clamps rather than overflowing, so `width: 100%` + padding gives a different used width under flex than under block. The chain `[1000, 1020, 960]` is internally inconsistent |
| **Indefinite axes and percentages** | percentages should resolve only against a definite size; the constraint model does not carry the distinction everywhere |
| **No real min/max-content pair** | intrinsic sizing computes one number where the algorithm wants two |

## What this page is not

It is not a roadmap and not an apology. Most of these are deliberate: failing
closed on `@container` is better than matching wrongly, and a CPU renderer that
approximates GPU effects by policy is better than one that pretends. The list
exists so that reading a type's presence in the re-export barrel does not mislead
you into thinking something runs.

If you need one of these wired, the gap is usually small and specific — most
entries above are "no caller", not "no implementation".
