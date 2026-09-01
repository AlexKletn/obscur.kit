# obscur.kit_v2 — one ledger

Seven agents swept the tree; adversaries then tried to break the load-bearing claims. What follows is what survived, deduplicated, ordered by what it costs to leave alone, and split three ways: **unfinished** (started, not done), **absent by decision** (with the reason, so nobody reopens it), and **rotten** (a doc, comment or test that is no longer true).

Every item carries a `file:line`. Effort is S (a day or less), M (a few days), L (a week or more, or it needs a decision before it needs code). "Done means" is stated for each, because an item nobody can tell is finished never gets finished.

Weighting: the stated targets are a Linux tablet where touch is primary, and an on-screen keyboard on a desktop layer surface. An item that bites an application author outranks one that offends a reader of the code.

---

## 0. Before anything else

**Nothing made since 2026-07-17 exists off this disk.**

`git log -1 origin/master` → `6f59a26f`, 2026-07-17. Local `master` is 108 commits ahead; `decomposition-campaign` (45 commits) has no upstream at all. On top of that sit 351 modified tracked paths and 680 untracked files, including `checks/layer-consumer/` in its entirety (a fixture `tools/xtask/src/consumer.rs:26-28` hard-codes and CI now runs) and 166 KB of audit prose (`docs/layer-consumer-audit.md`, `docs/perf-audit.md`) that is the only record of why ~50 files changed.

This is the only item in the ledger whose cost has no ceiling, and it is the only one that gets worse every day. Note the trap while doing it: 617 of the untracked files are `.doc/*.md` build inputs of *tracked, modified* `.rs` files (e.g. `crates/domain/obscur-style/src/computed/computed_style.rs:50` includes `computed_style.doc/remove.md`, which is untracked), so a partial commit produces a HEAD that fails `include_str!`. Commit with `-A`, by campaign, not by hunk.

On the split: 34 modified files carry more than one campaign, and three of them (`computed_style.rs`, `style_resolver.rs`, `property_compiler.rs`) mix the FastHasher swap, the copy-on-write custom-property change and the symbol interning **on the same line**. Land FastHasher + interning as one commit; everything else splits cleanly by path. Aiming for twelve clean commits will cost a day and end in a broken intermediate.

**Effort:** S. **Done means:** `git push origin master && git push -u origin decomposition-campaign`, and `git status` shows nothing untracked under `checks/`, `docs/` or any `.doc/` directory.

---

## Part 1 — Unfinished work that bites an application author

### 1. An authored `<button>` silently swallows every author rule on its children

`crates/frontend/obscur-ui-ir/src/name_binding.rs:200` maps the tag `button` to the built-in component; `crates/facade/obscur-kit/src/bootstrap/builtin_components.rs:26` registers `ButtonFactory` and `crates/facade/obscur-kit/src/app/app_parts.rs:113` installs it for both runners. `ButtonProps::contract()` (`crates/components/obscur-widgets/src/button/button_props.rs:27-38`) never sets a style scope, and `ComponentContract::new` defaults to `StyleScope::Scoped` (`crates/domain/obscur-view/src/component/component_contract.rs:48`). `crates/application/obscur-runtime/src/runtime/component_render_pass.rs:142-156` commits the slot-projected authored children as that host's own subtree — so everything an author writes inside `<button>` is a component internal.

Proved by execution, out of workspace, against the facade: with `.inner { color: rgb(200,10,10) }` installed through `CssPipeline`, `<div class="btn"><span class="inner">` computes `(0.784, 0.039, 0.039)`; `<button class="btn"><span class="inner">` computes `(0, 0, 0)`; the same button markup with the built-ins *not* registered computes `(0.784, 0.039, 0.039)`. Mounting the button is what closes the scope.

This is not a component-authoring subtlety. Plain hand-written markup — `<button class="btn"><span class="label">…</span></button>` — silently loses every author rule on the span, in every `App` and `App::layer` program. The workspace's own fixture has that exact shape (`checks/facade-consumer/src/counter.rs:23-25`). It is invisible because `::part()` and the UA sheet still cross, so the element computes *a* style, just not yours. Nothing tracks it: `docs/layer-consumer-audit.md`'s finding 2-K describes the symptom with the wrong cause and has no row in that file's own status table.

**Effort:** M — the fix is a rule, not a patch. Content projected through a slot was authored *outside* the host, so it should keep its author's scope (the shadow-DOM rule for slotted content); alternatively the shipped controls that project author content declare `StyleScope::Open`. **Done means:** a facade test that styles a `<span>` inside an authored `<button>` from a document sheet and asserts the colour lands, plus the same for `<checkbox>`, `<radio>`, `<select>`, `<input>`.

### 2. A text field's value never reaches an `App`

`crates/application/obscur-runtime/src/runtime/ui_runtime/event_listeners.rs:29,35,50` — `on_text_input`, `on_value_input`, `on_change` — have no counterpart in `App`'s method list (`crates/facade/obscur-kit/src/app/app.rs:80-302`), and `EventPayload` has only `Empty`/`Pointer`/`Key`/`Wheel` (`crates/domain/obscur-interaction/src/event/event_payload.rs:8-18`), so the widened `on_command` cannot carry text either. Every example that reads a field owns its own loop (`examples/controls_window.rs:130`, `showcase_window.rs:134`, `wireframe_live_window.rs:146`); zero `App` examples do.

`docs/api/02-view-model.md:141` tells the author "Binding `EVENT_INPUT` is what makes a node a text field". They do that, get a field that edits on screen, and hear nothing back. The only route is `control.with_runtime(|rt| rt.on_text_input(…))` — the escape hatch as the main road, for the first form anybody writes.

**Effort:** S. **Done means:** `App::on_text_input` / `on_change` / `on_value_input` exist, an example under `examples/` reads a name field through `App` alone, and `docs/api/02` stops naming handlers `App` does not have.

### 3. There is no clipboard, on any platform

`crates/application/obscur-runtime/src/runtime/key_edit_resolution.rs:91-97` resolves only `a`, `z`, `y` under Ctrl/Meta; everything else is `KeyEdit::Declined`. `crates/domain/obscur-text/src/editing/edit_command.rs:19-48` has no `Copy`/`Cut`/`Paste` variant, although its own doc file claims "a platform keymap resolves key chords, clipboard actions, and IME events into these commands". `ClipboardProvider`/`MemoryClipboard` (`crates/domain/obscur-data-transfer/src/clipboard/`) are named outside their module only by the facade barrel and one spec test; `rg 'impl ClipboardProvider'` finds one impl, inside the domain crate. No `arboard`, `smithay-clipboard`, `wl-clipboard` or `copypasta` anywhere in any manifest, and no `set_clipboard_provider` beside the ~30 other port installers on `UiRuntime`.

Not a decision: §6.20 (`specs/obscur-kit-2-spec-drafts/06-platform-windows-and-services.md:349`) requires copy/cut/paste integration; the only recorded non-goal nearby is clipboard *history*. The drag half of the same domain crate is fully wired, which is what makes this look dropped rather than deferred.

One correction to the obvious framing: an application author *can* bolt on copy — `editing_text()`, `editing_selection()` are `pub` and Ctrl+C reaches the app's own command router. Paste is where it breaks: the only public write is `set_editing_text` → `TextEditSession::reseed` (`crates/application/obscur-runtime/src/runtime/text_edit_session.rs:52-67`), which rebuilds the buffer, forces the caret to the end, resets `scroll_left` and discards undo. A selection-anchored paste is impossible from outside; `apply_edit` is `pub(super)`.

**Effort:** M — domain command variants, a resolver arm, a runtime port, a Wayland/X11 adapter. **Done means:** Ctrl+C/X/V work in a shipped `<input>` and `<textarea>` on Linux, paste replaces the selection, undo survives it, and the port has one real adapter plus the existing `MemoryClipboard` for tests.

### 4. A stylesheet can react to width and height, and to nothing else — including reduced motion

`crates/frontend/obscur-css/src/compiler/condition_lowering.rs:149-157` lowers `min/max/plain width|inline-size` and `height|block-size`, then `// Boolean features, orientation, prefers-*, unknown names.` `_ => return None`. So `orientation`, `pointer: coarse`, `any-hover`, `prefers-color-scheme`, `prefers-reduced-motion` and `resolution` all fail closed. It is at least honest — `crates/facade/obscur-kit/src/bootstrap/css_pipeline.rs:69-70` turns each into a runtime diagnostic.

Worse, and separate: even a condition that *did* lower would not gate motion. `crates/frontend/obscur-css/src/compiler/stylesheet_compiler.rs:359-366` appends `inner.motion_rules` into the enclosing scope **unconditioned**, one line above where matchable rules get wrapped in `RuleCondition::All`, with a comment saying exactly that. So `@media (prefers-reduced-motion: reduce) { * { transition: none } }` is inert twice over.

On the tablet this is the row that hurts most in styling: no sheet can respond to rotation, and none can size hit targets for a finger versus a mouse. Reduced motion is an accessibility obligation, not a nicety.

**Effort:** M. `orientation` and `pointer`/`hover` need a media-environment field each; `prefers-*` needs a source (the system-theme appearance port already exists and already knows the desktop's preferences). The motion asymmetry is visible one line from its fix. **Done means:** `@media (orientation: landscape)`, `(pointer: coarse)` and `(prefers-reduced-motion: reduce)` each match, and a transition authored inside a false `@media` block does not run.

### 5. Nothing scrolls horizontally — and `overflow-x` still clips, so the content is hidden and unreachable

`crates/application/obscur-runtime/src/runtime/scroll_offsets.rs:24` is `offsets: FastMap<NodeId, f32>` — one scalar per node; `clamped`, `set`, `shift` all bound by `max_scroll_top()`, and `refold` (`:79-113`) accumulates `dy` only. `crates/application/obscur-runtime/src/orchestration/scroll.rs:14-30` carries `content_height` and no `content_width`. The whole public API is block-axis: `scroll_top`, `set_scroll_top`, `scroll_by(node, dy)`.

Four consumers sit behind that one scalar, not one:
- the wheel (`ui_runtime/scrolling.rs:145`, `delta_x` is delivered and dropped),
- **touch pan and fling** (`ui_runtime/gestures.rs:68` and `:128`) — so on the tablet there is no horizontal drag at all,
- `scroll_into_view` (`scrolling.rs:20-56` compares only `y`), so Tab to a node offscreen to the right reveals nothing — this reaches focus and accessibility,
- the scrollbar (`orchestration/scroll_container_paint.rs:34-65`), of which only a vertical one is ever painted.

And the trap: `crates/domain/obscur-style/src/computed/overflow_axes.rs:32-60` pairs the axes per css-overflow-3 §3, so `overflow-x: auto` computes `overflow-y: auto`, `clips()` is true, and the box clips on x while nothing can ever move it. Content is not merely unscrolled; it is invisible and unreachable.

One correction to the headline: single-line text fields *do* scroll horizontally, through an entirely separate `scroll_left` on the edit session (`runtime/text_edit_session.rs:19-20,97-104`, computed at `runtime/caret_scroll.rs:60-70`). That is a per-field text feature, not the scroll-container model.

Deliberately deferred and said so in three places (`scrolling.rs:140-142`, `scroll.rs:35-39`, `docs/api/12-reality.md:97`) — but no spec draft blesses a one-axis scroll model, so it is owed work, not a decision.

**Effort:** M. Widen `ScrollOffsets` to a `Point` and `ScrollContainer` to carry `content_width`/`max_scroll_left`; the four consumers follow. **Done means:** a wide table scrolls with the wheel, with a drag, with a fling, reveals a right-offscreen node on Tab, and paints a horizontal bar.

### 6. Safe-area and keyboard insets have no producer anywhere

`crates/application/obscur-runtime/src/runtime/ui_runtime/surface.rs:58,63,68` define `set_safe_area_insets`, `set_keyboard_insets`, `set_environment`. Every caller in the tree is a test (`crates/facade/obscur-kit/tests/environment_insets.rs`, `spec_ch6b_insets.rs`, `crates/application/obscur-runtime/tests/scroll_containers.rs:84,130,161`). Neither `app/runner.rs` nor `app/layer_runner.rs` mentions insets, and `App` has no door onto them either.

So on the tablet `env(safe-area-inset-*)` is permanently zero — content sits under the cutout and the rounded corners — and because keyboard geometry never arrives, a focused field is never lifted out from under the on-screen keyboard, even though the scroll-into-view machinery that would do it exists and is tested. §6.15's spec tests are green precisely because the test supplies by hand what production never supplies: the chapter is closed at the runtime seam and open at every seam that would feed it.

There is an irony worth naming: `../obscur.virtual-keyboard` is a kit application that occludes the bottom of the screen, and there is no channel by which it, or the compositor, could tell another kit application so.

**Effort:** S for the `App`/`AppControl` setter; M for sourcing real values from winit and from the layer surface. **Done means:** a notched device (or a fake reporting insets) shows `env(safe-area-inset-top)` non-zero in a computed style, and a focused field scrolls clear of a reported keyboard rect.

### 7. Touch stops at tap, pan, long-press and fling

Four gaps, one story, all on the primary-input target:

- **No gesture arena.** §2.4 is normative — "Competing recognizers use a gesture arena" (`specs/obscur-kit-2-spec-drafts/02-components-tree-and-dsl.md:69`). `GestureArena` has zero production consumers; the real path is `crates/application/obscur-runtime/src/runtime/pointer_sessions.rs:7,51,62`, holding one `LongPressRecognizer` and one `PanRecognizer` per session with ad-hoc arbitration (a `press_discarded` bool at `:92`/`:257`). Every new gesture must be hand-wired against every existing one.
- **No pinch, no zoom.** `GestureKind::Pinch` is declared (`crates/domain/obscur-interaction/src/gesture/gesture_recognizer.rs:34-35`) and the gesture directory holds only `fling.rs`, `long_press_recognizer.rs`, `pan_recognizer.rs`. Nothing named zoom/fit-width/min-scale exists outside two examples' Ctrl+= shortcut. All of §4.10 is unreachable.
- **No back.** `grep -rniE "predictive.?back|semantic.?back|exclusion.?region|edge.?gesture"` over `crates/` returns two prose mentions. A tablet app has no back affordance of any kind — no gesture, no event, no route hook — and the toolkit cannot tell a compositor which edges it wants. (§6.13's `Router` already holds the history stack a back gesture would pop; wiring §6.16 and §6.13 is close to one task.)
- **No overscroll, snap, chaining or anchoring.** `overscroll`, `scroll-snap-*`, `scroll-margin`, `scroll-padding`, `overflow-anchor` return nothing anywhere in `crates/`. So there is no rubber-band at the ends, a paged or carousel UI cannot be built, and a nested scroller that hits its end swallows the gesture instead of handing it to its parent.

**Effort:** L, and it is four separate pieces. Rank inside it: arena first (everything else needs somewhere to land), then overscroll+chaining (users read their absence as "this isn't a native app"), then pinch, then back. **Done means:** two recognizers can contend and one can yield; a nested scroller at its end passes the gesture up; two fingers scale a node; a back gesture emits an event an application can bind.

### 8. An `App` publishes no accessibility tree, at any feature combination

`install_accessibility` exists at `crates/facade/obscur-kit/src/bootstrap/kit_bootstrap.rs:258` and its only caller is its own test at `:652`. Nothing under `crates/facade/obscur-kit/src/app/` mentions `accesskit` at all — and `accesskit` is in the `full` feature (`Cargo.toml:55`), so turning the feature on changes nothing. The installer's own doc states the consequence: "Not calling this leaves the runtime's honest-capability default: no semantic tree is built at all."

The written reason for the gap — "AccessKit's platform half needs an OS window, which this kit never owns" (`kit_bootstrap.doc/install_accessibility.md`) — is true of `KitBootstrap` and false of `App`, which owns a winit `Window` at `app/runner.rs:448-484` and could finish the path with `accesskit_winit`. The reason was never re-examined after `App` arrived. Meanwhile `docs/api/05-components.md:109,116` and `06-input.md:315` describe "the accessibility projection announces…" unmarked, and `README.md:33` says an unmarked API is wired.

**Effort:** S–M. **Done means:** an `App` built with `--features full` is visible to Orca on Linux, and the row in `docs/api/12-reality.md:95` stops reading as "works when you enable it".

### 9. Every press, every release, and every hover entry restyles the whole tree

Measured today with an out-of-workspace probe counting the `style.resolved` telemetry counter over an 83-node tree: build frame 83, idle frame 0, hover *within* a row 0, hover from one row to the next **78**, press **83 of 83**, release **83 of 83**. The incremental machinery is demonstrably live and it is bypassed on every interaction.

The cause is *not* the `:active` ancestor projection — that walk is required by Selectors-4 §9.2 (`specs/w3c/selectors-4.txt:2717-2725`) and must stay. It is that `RestyleScope::expand` (`crates/application/obscur-runtime/src/invalidation/restyle_scope.rs:41-45`) is purely structural and sheet-blind: a state change reaching the parentless host means "restyle everything", even when no rule in any sheet mentions `:active`. Its own unit test says so (`restyle_scope.rs:162 the_host_reaches_the_whole_tree`). Half the fix is built and unwired: `StyleInvalidation::note_pseudo_state_changed` and `StyleDependency::affected_by_pseudo_change` (`crates/domain/obscur-style/src/invalidation/style_invalidation.rs:87-99`, `style_dependency.rs:109-114`) have zero callers outside `obscur-style`.

`docs/layer-consumer-audit.md` measures the cost on the target device: 5.0 ms against 1.68 ms for a hover-only frame. Note the audit's own cross-reference to `docs/perf-audit.md` finding #1 is stale — that mechanism (a component re-render collapsing scope to `None`) was closed and is test-guarded; this reproduces in a tree with no components at all. Also note the 1.68 ms figure is the best case: hover between adjacent list rows costs 78 of 83 nodes, because `restyle_scope.rs:53-56` marks every following sibling's subtree in case a `~` selector exists.

**Effort:** L — it needs a selector-dependency map (which sheets mention which pseudo-classes and which sibling combinators) and an inherited-property diff. **Done means:** the probe above reports single-digit resolved nodes for a press, a release and a hover step in a sheet with no `:active` rule, and the numbers are pinned by a test in `crates/facade/obscur-kit/tests/incremental_restyle.rs`.

### 10. `position: sticky` establishes no stacking level, so a sticky header loses the click as well as the paint

`crates/application/obscur-runtime/src/orchestration/stacking.rs:157-166` — `is_positioned` matches relative, absolute, fixed. `POSITION_STICKY` exists (`crates/domain/obscur-style/src/property/layout_keyword.rs:187`) and is not in the list, while `obscur-layout`'s own `Position::is_positioned` (`crates/domain/obscur-layout/src/model/position.rs:88-92`) *does* count it. Two consequences, and the second is worse than the Servo catalog records: `own_stacking` returns `None` for a sticky node, so its `z-index` is ignored entirely — the author has no escape hatch — and because `crates/application/obscur-runtime/src/runtime/hit_testing.rs:130-172` keys off the same `effective_stacking`, a sticky header that visually covers a scrolled row also **loses the click to that row**.

Sticky is live, not a stub: mapped at `orchestration/layout_style_mapping.rs:228`, driving invalidation at `runtime/layout_classifier.rs:167-175`, with an end-to-end test at `crates/facade/obscur-kit/tests/position_css.rs:240`. Sticky headers over a scrolled list are exactly the case, and this is the cheapest real defect in the ledger.

**Effort:** S — three tokens in `stacking.rs:160-162`, plus a paint-order test and a hit-order test. **Done means:** a `position: sticky; z-index: 10` header paints above and receives the click over a scrolled row beneath it.

### 11. Hit testing and painting sort by different keys

`crates/application/obscur-runtime/src/runtime/hit_testing.rs:160-168` sorts by `(effective z, PaintLayer, logical_depth, preorder)`. The compositor sorts paint units by `(z, layer, tree_index, order)` with **no depth key** (`crates/application/obscur-runtime/src/orchestration/scene_coordinator.rs:620`, where `tree_index` is documented as pre-order document position). CSS 2.1 §9.9 layer 6 decides by tree order alone. So the two orderings that `hit_testing.doc/resolve.md` promises will not drift apart, drift at exactly this key.

The tie is reachable in ordinary layouts, not contrived ones: `stacking.rs:116` makes an unpositioned descendant inherit its positioned ancestor's `PositionedZeroOrAuto`, so a deep child of an earlier absolutely-positioned panel ties on both leading keys with a later overlapping panel's shallower content, and wins on depth while painting beneath. "I clicked the button and the panel behind it answered."

It is left standing on purpose and documented as a known defect, because removing the depth key turns the §4.7 oracle `ch4_7_a_the_hit_chain_is_the_logical_tree_not_the_portal_target` red — that fixture leans on the depth key precisely because portals project nothing (item 12). Verified: `cargo test -p obscur-kit --test spec_ch4b_hit_testing` is 6/6 green today, and green only because of that key.

**Effort:** S once unblocked; the block is a decision. **Done means:** either portals are wired and the key is removed, or the oracle is amended by the owner and the key is removed. It should not stay pending indefinitely — it is a wrong click today.

### 12. `<Portal>` in a `.okui` file compiles, validates, and does nothing

`crates/frontend/obscur-okui-compiler/src/validation/builtin_contracts.rs:22` declares `ComponentContract::new("Portal").requiring_prop("target")` and `crates/frontend/obscur-ui-ir/src/name_binding.rs:239` binds the tag, so `<Portal target="overlay"><div/></Portal>` produces *zero* diagnostics (`crates/facade/obscur-kit/tests/okui_child_contracts.rs:129` asserts exactly that: "an ordinary container"). `obscur_primitives::Portal` emits `PROP_TARGET` (`crates/components/obscur-primitives/src/portal.rs:18`) and nothing in production reads that symbol; `obscur_tree::portal::Portal::apply` has only its own test callers; `visual_parent()` is written by the reconciler and read by nothing outside tests.

Content renders in place — clipped and stacked by its ancestors. Silent wrong output with a green compile is the worst failure shape in the tree.

The Rust side was already made honest: `crates/components/obscur-primitives/src/portal.doc/Portal.md:14` has an "Inert as of 2026-08-07" section saying `PROP_TARGET` is read by nothing and a child will not escape an `overflow: hidden` ancestor. The honesty stops at the markup boundary: `name_binding.doc/PORTAL.md` is one line, `builtin_contracts.doc` says only "requires `target`", and `docs/api/09-authoring-formats.md:378` lists `Portal` among "Primitives (six)" with no caveat — in a table that flags `Slider`/`Progress` two rows down as "Not reachable from markup".

**Effort:** S for honesty (a compiler note on `<Portal>`, plus the three doc rows), L for the feature (target registry, visual child list, a reader, and a hit tester that currently walks `logical_parent()` for clips and so cannot be fixed in step). **Done means:** short term, an author who writes `<Portal>` is told it is inert; long term, its child escapes an `overflow: hidden` ancestor and the §4.7 oracle passes without the depth key.

### 13. `<VirtualFor>` drops its `each` — and emits Rust that does not compile

Three lowering paths all discard it. The bridge (`crates/frontend/obscur-okui-compiler/src/bridge/ui_ir_view_bridge.rs:101-110`) collects children into a template and calls `virtual_collection(DEFAULT_VIRTUAL_ROW_EXTENT, template)` without reading `each` or `key`. The build-time codegen short-circuits `<VirtualFor>` out of `emit_loop` — where `each` is read — into `emit_virtual_collection` (`crates/frontend/obscur-okui-compiler/src/codegen/view_codegen.rs:97-104,156-174`), which also never calls `emit_key`. The screen path deliberately stops the binding walk at a `<VirtualFor>` (`crates/facade/obscur-kit/src/screen/okui_screen.rs:170-174`).

So every realized row is identical and keyless — on the one construct the toolkit's origin use case is built on.

The part not previously recorded: the build-time path is worse than N identical rows. `NameResolver` pushes loop-local `item`/`index` for every loop including `<VirtualFor>` (`crates/frontend/obscur-okui-compiler/src/resolution/name_resolver.rs:66-78`) and nothing downstream refuses them, so `<VirtualFor each={rows}><span class={item.css}>{item.name}</span></VirtualFor>` compiles with `has_errors = false` and emits `item.css` / `item.name` into a template with no `for` binding — rustc then fails inside `OUT_DIR` or a macro expansion with no `.okui` source location.

Structurally, an author cannot even *name* a virtual list from markup: `ViewNode::virtual_list(list_id, count, row_extent)` (`crates/domain/obscur-view/src/view/view_node.rs:67`) is the only marker `VirtualLists::realize` can attach a registered builder to, and it is constructed only by hand-written Rust.

**Effort:** S for the diagnostic; M for an id-carrying marker attribute (`<VirtualFor list="files" count={…}>`) that lets a Rust row builder fill it — which closes the approved epic without inventing per-item codegen. **Done means:** a loop-local reference inside `<VirtualFor>` is a front-end error naming the file and line; and a `.okui` file can declare a virtual list that a registered builder fills with distinct rows.

### 14. `var(--x, fallback)` discards the fallback, at both ends, silently

`crates/frontend/obscur-css/src/compiler/value_lowering.rs:155-170` — `var_reference` is one `find_map` returning the first `--`-prefixed name; nothing else in the argument list is read, and the function takes no diagnostics sink, so the silence is structural (contrast `transition_compiler.rs:98-106`, which does take one).

It is also dropped on the *definition* side: `property_compiler.rs:84-85` compiles `--brand: var(--accent, #06c)` through `sole_custom_ref` into a bare `CustomRef(--accent)`, so the `#06c` never even reaches the `RawTokens` escape hatch that saves `--font-body: Inter, sans-serif`.

And the degrade is not "the property is missing". `substituted_value.rs:44-47` makes an unresolved `CustomRef` `fits() == false`, returning `Unset`, so `style_resolver.rs:308-310,344-348` computes the **inherited** value for an inherited property. `color: var(--missing, #f00)` paints the parent's colour — plausible on screen, no diagnostic to grep, harder to spot than absence.

The kit's own flagship feature instructs authors to write the broken idiom: `crates/facade/obscur-kit/src/system_theme/theme_vars.doc/ThemeVars.md:14-16` names `var(--obscur-sys-accent, #06c)` as "the guard an author would reach for" and admits the fallback is discarded — for a feature whose contract is "only what the desktop said", i.e. missing variables are the expected case. Zero tests pin any of it, and no ledger row tracks it.

The obstacle nobody wrote down: `PropertyValue::CustomRef` (`property_value.rs:68-69`) carries only an `InternedSymbol`, so modelling a fallback needs a new or boxed variant plus a `substitute` change. The `env()` path four functions down (`value_lowering.rs:191-204`) already consumes its fallback correctly, so the pattern exists in the same file.

**Effort:** M for the real fix; S for the stopgap (give `var_reference` the sink and report when the arg list holds anything past the first name). **Done means:** `var(--missing, 8px)` resolves to `8px`, `var(--a, var(--b))` chains, and a test pins both at a use site and in a custom-property definition.

### 15. `translate(-50%, -50%)` — the centring idiom — is refused

`crates/frontend/obscur-css/src/compiler/transform_lowering.rs:164-180` rejects a percentage from a `Percentage`, from a dimension, and from inside `calc`/`min`/`max`/`clamp`; the declaration becomes `Unsupported` at `property_compiler.rs:137`, and an in-file test pins it. There is no CSS Transforms 2 individual `translate` property either (`PropertyId::Translate` does not exist), so an author has no second spelling.

This is much improved — it used to lower to identity silently and now raises a diagnostic — but the pixels are still wrong for the most common transform anyone writes.

The cost of the fix is larger than `docs/api/12-reality.md:119` implies. It is not "fold the pending translate into `resolved_transform` and mirror it in `hit_testing`". `PropertyValue::as_transform` (`crates/domain/obscur-style/src/property/property_value.rs:319-324`) answers `None` for `TransformFunctions`, and the runtime contains **zero** references to that variant; `orchestration/paint_transform.rs:35-38`, `orchestration/arrange_pass.rs:22-29` (containing-block establishment) and `runtime/motion_driver.rs:649-651` (animated transforms) all read the flattened accessor. Relaxing the lowering alone would reintroduce the original defect — no movement — and this time without the diagnostic.

**Effort:** M. **Done means:** an absolutely-positioned box with `left: 50%; transform: translate(-50%, -50%)` is centred, is hit-tested where it is drawn, still establishes a containing block, and animates.

### 16. `tabindex` and `role` are unauthorable, so a custom composite cannot be reached by Tab at all

`crates/application/obscur-runtime/src/runtime/focus_order.rs:71` builds every stop as `FocusNode::new(node).focusable(true)` and never sets an index; `with_tab_index` (`crates/domain/obscur-interaction/src/focus/focus_node.rs:35`) has callers only inside test modules. So `TabOrder::compute`'s sort key is a constant and `is_tab_focusable`'s `tab_index >= 0` gate is constant-true.

The larger half, missed by every ledger row on this: ring membership is `SemanticTreeBuilder::role_of(...).is_interactive()` (`focus_order.rs:88`), and `role_of` (`semantic_builder.rs:256-266`) derives the role from `local_name` alone — there is no authored `role` override. A composite control built out of `div`s, which is the whole point of a headless view-builder toolkit, cannot enter the tab ring by any means.

Worse than absent: an authored `tabindex` is silently accepted and inert. `attr_symbol` (`crates/frontend/obscur-ui-ir/src/name_binding.rs:348-378`) falls through to `fallback_symbol`, so `<button tabindex="-1">` parses, hashes to an ordinary property, rides the whole tree and is read by nothing — the same trap the `placeholder` arm's own comment records as a past shipped bug, still open one arm over.

(One nuance: `disabled` *is* reserved and drops a subtree from the ring, so a control can be removed — at the price of also killing pointer interaction and applying disabled styling.)

**Effort:** M — a reserved `TAB_INDEX` and `ROLE` in `crates/domain/obscur-view/src/view/style_attribute.rs`, mirrored in `name_binding.rs`, read back in `FocusOrder::collect`. The mirror is why it is not a one-liner; `okui_name_binding_drift.rs` exists because those two have drifted before. **Done means:** `tabindex="-1"` removes an operable control from the ring, a positive index reorders it, and `role="button"` on a `div` puts it in.

### 17. Flex clamps a content-derived border box where block does not

`crates/domain/obscur-layout/src/algorithm/flex_layout.rs:400-401` (single-line) and `:605` (multi-line) form `border_box = own_w + bp_x` and pass it through `BoxConstraints::constrain`, which clamps to `max`. `flow_layout.rs:481,699` and `grid_layout.rs:661` use `constrain_allowing_overflow` instead — grid's comment at `:648-660` fixes it for grid with exactly this reasoning.

Measured by running the two algorithms with identical styles (`width: 100%`, 20 px horizontal padding, one `width: 100%` child, 1000 × 600 offer): flow reports 1040 × 400; flex reports **1000** × 400 while placing a 1000-wide child at x = 20 — a right edge of 1020, outside the box it reports. It is not merely under-reporting; it is internally inconsistent.

Only flex is a live instance. `docking_layout.rs:174`'s clamp is structurally inert (every band is bounded by the content box), and `custom_layout.rs:69` cannot bite on the inline axis, reads no authored width or height, and has no runtime consumer — the runtime dispatches only Flow, Flex and Grid (`crates/application/obscur-runtime/src/orchestration/formatting_context.rs:19-23`).

Nothing pins it, on purpose: `crates/facade/obscur-kit/tests/percentage_in_padded_parent.rs:110` uses `width: 800px` for the flex case, and its doc file says in writing that the fixed width keeps this defect out of the test.

**Effort:** S — two call sites, using grid's just-landed comment as the template. **Done means:** the `width: 100%` + padding case gives 1040 under flex as it does under block, pinned by a mirror of the block test in `percentage_in_padded_parent.rs`.

### 18. Overflow clips to the border box in four places, and to the outer curve when rounded

`orchestration/scroll.rs:101` (`let scrollport = b.bounds;`), `orchestration/scene_coordinator.rs:213-215` (the clip) and `:237` (the scrollbar rect — the fourth site, previously uncounted), and `runtime/hit_testing.rs:308-315` (the ancestor clip) all use the border box. Layout genuinely insets children by border + padding (`flow_layout.rs:71-74`), so the clip is loose by the border width.

Two corrections to the recorded version. First, the border width is **not** unavailable: all four sites already hold `&ComputedStyles`, `orchestration/border_paint.rs:127` already resolves per-side widths from exactly that, and `orchestration/arrange_pass.rs:32` already defines `padding_box(border_box, border)` and uses it at `:229`. The fix is a shared helper at four call sites, not threading border widths through `LayoutBox` and every layout algorithm. Second, the rounded case is worse than "bounded by the border width": `scene_coordinator.rs:213` clips to the *outer* border curve where CSS clips to the padding edge with radii reduced by the border width, so a thick rounded border — cards, buttons — shows a visible crescent of bleeding content at each corner.

Adjacent, same family: `scroll.rs:59-79` computes content height as `max child bottom − scrollport top`, so a scroll container's own `padding-bottom` is cut off at the end of the scroll.

The four-way agreement is deliberate and documented, and it must be kept — narrowing one site alone makes the pointer disagree with the pixels.

**Effort:** S–M. **Done means:** all four sites call one `padding_box_of(computed, node, bounds)`, the rounded clip uses reduced radii, and a test asserts a child does not paint over a 10 px border.

### 19. SVG drops `fill-rule` and `fill-opacity`

`crates/adapters/obscur-resource-decode/src/decoder/svg_vectorizer.rs:130` builds `Path::new()` and never calls `with_fill_rule`, so every shape gets `NonZero` even though the scene model carries the concept (`crates/domain/obscur-scene/src/path/path.rs:26`) and both backends honour it. `:118-127` hardcodes alpha to 1.0 and never reads usvg's `Fill::opacity()`. Neither is covered by the documented scope note in `scene_vector.doc/SceneVector.md`, which lists gradients, patterns, masks, filters and text.

So every even-odd path — a ring icon, a donut, a counter-shaped glyph, the letter O — fills solid, and `fill-opacity="0.5"` renders opaque. Silently wrong pixels on the most ordinary asset a UI ships.

(One nearby lead was refuted and should not be reopened: group transforms are *not* dropped — usvg pre-resolves ancestor transforms into absolute path data, per `usvg-0.46.0/src/tree/mod.rs:1384-1387`.)

**Effort:** S — fill-rule is close to a one-liner; opacity is one multiply. **Done means:** a ring SVG shows its hole and a half-opaque fill is half opaque, pinned by a CPU-render test.

### 20. `visibility` does nothing, and `revert` is not lowered

Two small holes in the CSS surface, both silent.

`PropertyId::Visibility` has **no consumer anywhere**: `grep -rn "PropertyId::Visibility\|VISIBILITY_" crates/ --include='*.rs'` finds only style-crate plumbing (`computed_property_set.rs`, `paint_only.rs`, `style_scope.rs`) and `longhand_catalog.rs:49`. `visibility: hidden` parses, cascades, computes, and hides nothing.

`revert` is not handled by `css_wide_keyword` (`crates/frontend/obscur-css/src/compiler/value_lowering.rs:12-23` covers `inherit`/`initial`/`unset` only), so it falls through `lower_keyword` and becomes an arbitrary interned keyword symbol — the one CSS-wide keyword where an author gets a genuinely wrong answer rather than the right one by another route.

Note what is *not* here. `initial`/`unset` on the ~25 keyword properties whose `initial_value_of` returns `None` (`crates/domain/obscur-style/src/computed/initial_value.rs:177-320`) are **not** silently broken (with one counterexample: `align-content`, whose fallback is `flex-start` where CSS says `stretch` — Pass 2 item A6): the author's declaration still wins the cascade, the property is left absent, and every consumer's fallback arm is the CSS initial — verified for display, box-sizing, direction, align-items, flex-wrap, text-align, text-transform, white-space, overflow-wrap and word-break. `overflow: initial` really does undo a UA `overflow: hidden`. The one divergence is `display: initial`, which yields Block where CSS says inline — a collision with the kit's absent inline box, not a defect.

**Effort:** S each. **Done means:** `visibility: hidden` stops the node painting while keeping its layout box; `revert` lowers to a `PropertyValue::Revert` (or is explicitly reported unsupported rather than silently mis-lowered).

### 21. The IME candidate window is never placed, and input hints reach nothing

The composition lifecycle *is* wired, contrary to what the adapter suggests: `crates/facade/obscur-kit/src/app/runner.rs:693-704` maps winit's `Ime::{Enabled,Preedit,Commit,Disabled}` onto `CompositionEvent`, with `set_ime_allowed` toggled at `:417` and `:484`. Two narrower things are missing. `set_ime_cursor_area` appears nowhere in `crates/`, against §5.12's "candidate-window placement uses caret geometry" — so composing CJK puts the candidate list at the window origin instead of under the caret, for every character. And `InputHints`/`InputPurpose` have zero consumers outside `crates/domain/obscur-text/src/ime`, including `InputPurpose::Password`/`Pin` and their `is_sensitive()` at `input_purpose.rs:44-45`.

The hints matter for the second stated target: purpose is exactly what tells an on-screen keyboard to show digits for a PIN, or to suppress autocorrect and the preview for a password. Today every field asks for the same generic keyboard.

**Effort:** S for the cursor area (the runtime already computes the caret geometry); M for hints, which need a channel to the compositor's text-input protocol. **Done means:** the candidate list follows the caret, and a `type="password"` field reports a sensitive purpose that a keyboard adapter can read.

### 22. Five capabilities the runtime has, that `App` has no door onto

`App` is the documented one-call entry point, and every one of these forces an author into `control.with_runtime(…)` or into abandoning `App` entirely. All four external consumers open their `main` with a registration loop for this reason.

- **Registering a component** — the toolkit's own extension point. `checks/facade-consumer/src/badge.rs:76`, `checks/layer-consumer/src/panel.rs:62`, `../test-app/src/app/mod.rs:53`, `../obscur.virtual-keyboard/crates/keyboard-app/src/main.rs:770` all call `runtime.register_component(...)` inside `with_runtime`. `App` has no `component()` method, though the queue is already drained before the first frame (`app_parts.rs:118`).
- **Waking the loop from another thread.** `App::waker` is `#[cfg(feature = "shell-layer")]` and takes a `LayerWaker` (`app/app.rs:126-131`); `AppControl` is `Rc<RefCell<…>>` and cannot cross a thread. The winit `EventLoopProxy` that would do it is constructed at `app/runner.rs:123`, is `Send`, and is private. So a windowed app with a directory scan, a network poll or an inotify watch has no supported way to say "rebuild".
- **Controlling its own window.** `app/runner.rs:712`'s `CloseRequested => event_loop.exit()` is the only exit; the `winit::window::Window` is private. No quit, no title change, no resize, no fullscreen — while the layer path has a full 141-line `LayerControl`.
- **Installing the system theme.** `install_system_theme` (`kit_bootstrap.rs:95,130`) has no caller under `src/app/`, so `--obscur-sys-*` is absent from every shipped application and `var(--obscur-sys-accent)` resolves to nothing. `App` *is* the host in this entry point, so the doc's "a host installs them" framing does not cover it.
- **Serving DevTools.** `docs/api/11-diagnostics.md:41-49` starts from `inspect_source_from(&rt)` — a `&UiRuntime` an `App` consumer never holds — and continues into a blocking `accept`/`serve` that cannot live inside `with_runtime`. Both in-repo hosting examples own their loop.

**Effort:** S each, M for the bundle. **Done means:** `App::component`, a `Send` wake handle for the windowed path, a `WindowControl` mirroring `LayerControl`, `App::system_theme()`, and one `App`-based DevTools example.

### 23. The scrollbar is a rectangle

`crates/application/obscur-runtime/src/orchestration/scroll_container_paint.rs:31-41` fills a track and a thumb from hardcoded constants (`SCROLLBAR_WIDTH = 8.0`, `SCROLLBAR_THUMB_COLOR = rgba(0,0,0,0.35)`). The parts are exposed to DevTools (`runtime/tree_inspector.rs:178` names `scrollbar-track` and `scrollbar-thumb`, satisfying §4.16), but no hit-test or pointer path mentions them. So a mouse user cannot grab the scrollbar of any container in any kit app, no stylesheet can restyle it — 35 % black everywhere, including on a dark theme — and there is no horizontal bar at all (item 5).

**Effort:** M. **Done means:** dragging the thumb scrolls, and `::part(scrollbar-thumb)` restyles it.

### 24. Five layout observers are built and none is driven

`crates/domain/obscur-layout/src/observer/` — 695 lines, 24 unit tests. `ResizeObserver`, `LayoutObserver` and `TransformObserver` have zero references anywhere in `crates/` or `tools/`; `IntersectionObserver` and `VisibilityObserver` are named only by `crates/facade/obscur-kit/tests/spec_ch4b_observers.rs:27-28`, which constructs them itself and feeds them rects it computed — so §4.4 reads green while nothing in the runtime constructs any of the five. §4.4's actual requirements (callbacks after layout commit, coalescing, scheduling the next transaction, no recursive mutation, scoped auto-cleanup) do not exist. `docs/api/12-reality.md` contains zero occurrences of "observer" — its own method would have caught this and was never run for this module.

No lazy-loading, no visibility callbacks, no resize callbacks, in a toolkit whose stated origin use case is large virtualized lists. `IntersectionObserver` is the one that pays for itself first, and the memory note records "scroll-chain geometry + observer driver" as an owner-approved build, so the decision to fill this already exists.

**Effort:** L. **Done means:** an application registers an `IntersectionObserver` on a node and is called after layout commit when it enters the viewport, with coalescing and cleanup pinned by tests.

### 25. Theming: three presets in a drawer

`obscur-theme-system` delivers exactly one thing to the workspace — its UA stylesheet string (`crates/facade/obscur-kit/src/default_stylesheet.rs:9,183`). Its other 601 lines — `ThemeToken`, `TokenValue`, `ThemePreset` with working `light()`, `dark()` and `high_contrast()` seeding real colours and spacings, `ThemeAsset` — have zero references in the workspace, including zero tests outside their own files. There is no function anywhere turning a preset's tokens into CSS custom properties.

Real theming happens through a different mechanism entirely (the `--obscur-sys-*` properties from the desktop appearance port, itself uninstalled on the `App` path — item 22), so `ThemePreset` is a second spelling that lost. `high_contrast()` is an accessibility feature sitting in a drawer.

**Effort:** M — a `ThemePreset -> Stylesheet` lowering emitting each token as a custom property, plus an `App` door. **Done means:** `App::theme(ThemePreset::high_contrast())` changes what is on screen.

### 26. A cluster of narrower confirmed limits

Each is documented at its own site with a reason; none is worth its own campaign, all are worth a reader knowing.

- **Locale-aware number, date and currency formatting is unbuilt** (`crates/domain/obscur-localization/src/lib.doc/_module.md:36-45`): `MessageValue::integer` renders ASCII digits with no grouping in every locale, so a Russian or French user sees `1234567` in an app whose *plural agreement* around that number is already correct. The reason (per-locale CLDR pattern vendoring is a different order of problem) is sound; the mismatch is unusually visible. M–L.
- **RTL is not reordered in caret geometry** (`crates/application/obscur-runtime/src/orchestration/caret_geometry.doc/line_caret_x.md:15`) and base direction is unwired because "a half-wired base direction is worse" (`text_attributes.doc/direction_for.md:20`). Arabic and Hebrew place the caret wrongly. M.
- **`fit-content(<length-percentage>)` lowers to no sizing keyword at all** (`crates/domain/obscur-style/src/property/layout_keyword.doc/SIZING_FIT_CONTENT.md:6`). S.
- **Flex has no `order` reordering and no baseline alignment** (`crates/domain/obscur-layout/src/algorithm/flex_layout.doc/FlexLayout.md:62`); `AlignItems` has no `Baseline` variant, so `align-items: baseline` is dropped as invalid and the item keeps `stretch` — which is what a browser does for a non-baseline item, so it reads as a missing feature, not a layout bug. M.
- **`justify-content: stretch` and `normal` are not accepted by the parser** (`crates/domain/obscur-style/src/property/layout_keyword.rs:310-320`) even though the layout model implements `Stretch` and `align-content: stretch` parses fine. The rendered result is right by accident (the initial value already stretches auto tracks), but the declaration cannot override an earlier `justify-content: center`. S.
- **`grid-template` / `grid` shorthands do not exist**; `grid-template-areas: none` becomes an Unsupported diagnostic; `grid-column: main 2` drops the whole shorthand as invalid (`crates/frontend/obscur-css/src/compiler/grid_placement.rs:8-28`); area-implicit named lines (`foo-start`/`foo-end`) are never synthesised; subgrid is absent. S–M each, subgrid L.
- **Border dashes do not fit their side**: `dashed_border_pattern` is a fixed `[3w, 3w]` with no side length in scope (`crates/application/obscur-runtime/src/orchestration/border_paint.rs:234-238`), so dashes do not meet at corners. S, cosmetic.
- **The CPU renderer's image path models no source sub-rectangle and no nine-slice** (`crates/adapters/obscur-render-cpu/src/renderer/cpu_renderer.doc/_module.md:66`). M.
- **`RenderExtension` is a 338-line island**: a host that builds a `CustomPass` at `PassStage::Overlay` gets a compile, a passing test, and no pixels (`crates/domain/obscur-render/src/extension/render_extension.rs:7`, zero references; neither backend reads an extension). Silent failure at the exact place an ambitious host is working. S to delete, M to wire.
- **`FramePipeline` is a second frame pipeline in the prelude** (`crates/application/obscur-runtime/src/frame/frame_pipeline.rs:21`, exported at `prelude/prelude.rs:27`) that no production code runs: no diagnostics sink, no components, no input dispatch, no motion. A test in the tree states it plainly (`okui_slot_projection.rs:539`). It is the first pipeline a user meets, and it already caused a false row in the reality ledger. S — take it out of the prelude.
- **Nine to fifteen empty `pub struct X;` skeletons are public API of the shipped facade**: `obscur_kit::geometry::space::{LocalSpace,ParentSpace,PhysicalSpace,ScreenSpace,WindowSpace}`, `runtime::quality::{AdaptiveQuality,QualityLevel,QualityPolicy}`, `runtime::lifecycle::{RuntimeLifecycle,ShutdownRequest,ReleaseResult,ShutdownConcern}`, `runtime::orchestration::{RenderCoordinator,TextCoordinator}`, `capability::event::{CapabilityChanged,CapabilityRevoked}` — each documented as "this stub exists to materialise the target source layout". Being `pub` in a `pub` crate is exactly why `cargo check` cannot warn. A reader concludes there is coordinate-space discipline to opt into, and a shutdown protocol. S — `#[doc(hidden)]` or `pub(crate)`.
- **The `LoopWaker` port has no adapter**: `crates/domain/obscur-windowing/src/port/loop_waker.rs:4` is implemented only by a test `Counter`, while winit ships its own unrelated `EventLoopWaker` and the layer path a third `LayerWaker`. Small in lines, but it is the seam an async resource load or a plugin would need. S.
- **Five `obscur-system` ports have no adapter on any platform** — `NetworkService`, `DocumentService`, `CaptureProvider`, `PowerProvider`, `PrivacyPolicy` — while four siblings (notification, dialog, shell, appearance) have two each. The `PowerProvider` gap pairs exactly with the `quality/` stubs: the input an adaptive-quality policy would read and the policy that would read it are both absent, which is why neither's absence is visible from the other. Capture at least returns `CaptureResult::Unsupported`, which is what §6.17 asks for. M–L.
- **`obscur-text`'s declared ports are not the seam anyone uses**: `impl FontProvider` has exactly one implementor, `MockFontProvider`, inside the domain crate; the fontconfig adapter implements no domain trait. A host wanting to supply its own fonts (the WASM path, the bundled-fallback epic) discovers that implementing the port achieves nothing. `TextSpan` being dead means rich text has a type and no path. M.

---

## Part 2 — Unfinished work that bites the project

### 27. The `App` entry point's own tests never run, and neither do twenty examples

`crates/facade/obscur-kit/src/lib.rs:10-11` gates `pub mod app` on `all(feature = "render-vello", feature = "surface")`; neither is in `default` (`Cargo.toml:33-42`); CI's only test job is `cargo test --workspace`, i.e. default features (`.github/workflows/ci.yml:41-49`). Verified: `cargo test -p obscur-kit --lib -- --list` prints 70 tests and not one from `app::`. The 25 excluded tests are in `app/app.rs`, `app_error.rs`, `layer_control.rs`, `layer_loop_stats.rs`, `turn_clock.rs`, `runner.rs` — and two of those files exist *because* of shipped user-visible defects: `runner.doc/now_micros.md` records that every pointer event went out stamped `0`, so `0 - 0 <= 500ms` made every second click a double-click; `turn_clock.rs` pins a motion clock that handed out 30 ms of animation for 20 ms of wall time. Both fixes are now guarded by nothing that runs. All 25 are pure logic — no display, no GPU.

Separately, 21 of 49 `[[example]]` targets are unsatisfiable under `default` and CI builds exactly one of them. Unbuilt: `hello_app`, `calculator`, `file_manager`, `panel`, every GPU proof (`blur_proof`, `backdrop_proof`, `color_filter_proof`, the gradient proofs), `showcase_window`, `uikit_window`. Checked today: `cargo check -p obscur-kit --features full --examples` exits 0. **That sentence used to end "so nothing has rotted yet — this is risk, not decay", and Pass 2 struck it by running them**: `margin_collapse_proof` and `overflow_proof` have panicked at *runtime* since 2026-07-21 while the compile check stays green (item A11). A compile gate cannot see this class, which is exactly why the CI gap costs more than the old wording conceded.

**Effort:** S. **Done means:** the test job gains `cargo test -p obscur-kit --lib --features full` and the features job gains `cargo check -p obscur-kit --features full --examples` plus `cargo build -p obscur-kit --no-default-features --features render-vello,window-winit` (the ordinary desktop configuration, currently green and built by nothing).

### 28. The clippy gate does not exist, and the baseline has drifted twice unnoticed

`.github/workflows/ci.yml:36-39` says "Warnings are already denied workspace-wide via `[workspace.lints]`" and then runs `cargo clippy --workspace --all-targets` with no `-D warnings`. `[workspace.lints.rust]` denies exactly one lint (`unsafe_op_in_unsafe_fn`); everything else is `"warn"`. The comment also claims the stub types are allowed there — nothing allows them; all 19 warn. Measured: `docs/perf-audit.md:24` recorded 359 on 2026-08-12 against a stated 359–360 band; today it is 366 warning lines / 244 distinct diagnostics. Seven warnings walked in past a gate everybody believes is closed.

The triage, in order: 6 unused imports and 2 unused `mut` are leftovers of the uncommitted campaigns — clear them before committing. About 44 (`needless_raw_string_hashes` 17, `redundant_closure` 20, `doc_lazy_continuation` 7, `collapsible_if` 4) are `clippy --fix`-able in one pass, mindful of CLAUDE.md's `--fix`/`cfg(test)` warning. The 71 `cast_*` deserve a workspace allow with a written reason — the manifest already allows `cast_precision_loss` on exactly the argument that covers the other three. The 19 `pub(crate)` skeleton stubs should carry `#[expect(dead_code, reason = "skeleton for the spec-defined entity")]`, which makes the decision machine-readable and makes the CI comment true. That lands the baseline near 120 and makes `-D warnings` affordable.

**Effort:** S–M. **Done means:** the clippy job actually fails on a new warning.

### 29. CI advertises four jobs it does not have, and the spec mandates three more

`.github/workflows/ci.yml:3-5` states the periodic jobs are "Miri, sanitizers, full fuzzing, GPU/OS matrix, WASM, MSRV, advisories". The file has eight jobs: fmt, clippy, test, architecture, features, advisories, msrv, wasm. No Miri, no sanitizers, no fuzzing, no GPU or OS matrix. `specs/obscur-kit-2-crate-architecture/05-cargo-policies-and-implementation-order.md:255-263` lists as **per-PR**: `trybuild`, snapshots, and fuzz corpus smoke tests for parsers. `grep trybuild|nextest|insta` across every manifest: no hits. No `fuzz/` directory. No stored snapshot or golden anywhere.

This is not theoretical. Two of the last twenty commits are hand-found parser denial-of-service bugs — `5d02cb2b` "a stray `}` is one fault, not an endless loop that eats the machine" and `8529f76f` "a closepath followed by a number is refused, not repeated forever" — exactly what a CSS / `.okui` / SVG-path fuzz corpus finds in seconds and a reviewer finds by luck. And the `okui!` macro's §9.1 rejections (assignment, `await`, closures, `unsafe`, `?`, loops, free calls) are tested as validator return values, never as the compile errors a user sees, because there is no `trybuild`.

The GPU/OS matrix is the one that matters for the stated ARM/Adreno target, and it is also the answer to the next item.

**Effort:** M. **Done means:** either the jobs exist or the sentence is shortened to the truth. A parser fuzz corpus and a `trybuild` suite are the two with the highest yield.

### 30. The GPU effect passes are proven by a CPU reimplementation living inside the test module

`crates/adapters/obscur-render-vello/src/effects/gaussian_blur.rs:622-712` — all four tests call `srgb_to_linear`, `linear_to_srgb` and `blur_at` defined at `:625-657` **inside `mod tests`**, a Rust port of the WGSL. The doc says so: "the GPU render `examples/blur_linear_light_proof.rs` proves the shader itself" — an example CI never compiles and nothing ever runs (item 27). Zero tests in `color_matrix.rs` (197 lines), `segmented_compositor.rs` (188), `filter_layer_pass.rs` (163), `premultiply_blit.rs` (131), `shadow_painter.rs` (137), `image_painter.rs` (127). The three `.wgsl` files reach a validator only through `device.create_shader_module`, so nothing checks them offline — and `premultiply_blit.wgsl` is modified in the working tree right now. The single test that rasterizes through Vello (`crates/facade/obscur-kit/tests/fractional_edge.rs:246`) is `#[ignore]`d and self-returns without an adapter.

`filter: blur()`, `backdrop-filter` and the colour-matrix filters can each break — a plain WGSL syntax error included — with the whole suite green, and the Rust mirror and the shader can drift with nothing to notice.

**Effort:** M. **Done means:** a software-adapter job (lavapipe/llvmpipe) that un-ignores `fractional_edge` and runs three of the vello proofs, turning the largest untested adapter in the tree (5,402 lines) into something with a floor.

### 31. `xtask` has no cross-crate reachability rule — the one mechanical fix `debt-closure` asked for

`tools/xtask/src/rule/` holds seven rules: `dependency_direction`, `dependency_graph`, `file_size`, `forbidden_file_name`, `main_entity`, `module_layout`, `unsafe_boundary`. `dependency_graph` *is* cross-crate, but only at the manifest level — `check.rs:17-24` parses member `Cargo.toml` files and runs layering, facade and cycle checks; nothing in it opens a `.rs` file or knows a symbol name. `grep -rn "reachab" tools/` returns one hit, in an unrelated crate.

The proof `debt-closure.md:171` offered is still live: `crates/domain/obscur-layout/src/pass/` still exists, `MeasurePass` and `ArrangePass` still have zero references outside that directory, and `LayoutPassResult`'s only outside reference is `invalidation/propagation_result.rs:3,26` — code that is itself unreachable. All of it passes `xtask check` today. The nearest existing mechanism, `unreachable_pub = "warn"`, is a within-crate *visibility* lint and structurally cannot see a `pub` item nothing downstream calls.

This is the item that stops the ledger decaying again, which is why it outranks its apparent size: this very sweep found two large dead islands (`obscur-tree/src/transaction/`, the `obscur-motion` driver tier) that no ledger names, because a grep can only confirm a symbol you already suspect.

There is now a precedent for its shape in the uncommitted tree: `consumer.rs` grew a `ConsumerProfile { rule, manifest, watched }` with a short `watched` path list, and `consumer.doc/watched.md` explains the constraint any reachability rule will hit too — the workspace carries a warning baseline in the hundreds, so it must be an allowlist of paths, not `-D warnings`. It will also need an escape hatch for types constructed only by spec oracles.

**Effort:** M. **Done means:** `xtask check` reports a `pub` item in `crates/{domain,application}` with no reference outside its own crate, with a documented allow list, and the `obscur-layout/src/pass/` island shows up in its output.

### 32. `read-fonts` and `font-types` are compiled twice, against two comments saying they are not

`Cargo.toml:76-80` — "Kept at harfrust's exact requirement so the two resolve to one build" and "Pinned to the version already resolved in the lockfile so it shares one build". `Cargo.lock` has read-fonts 0.39.2 *and* 0.40.2, font-types 0.11.3 *and* 0.12.1; harfrust 0.10.0 wants 0.40.2, skrifa 0.42.1 wants 0.39.2. Two full font-table parsers in every build and in each of the 178 test binaries — a direct contributor to the 8.2 GB `cargo test --workspace` peak this box has to work around. Worse, the comments actively mislead whoever next tries to bump a pin.

**Effort:** S — check whether a newer skrifa takes read-fonts 0.40. **Done means:** one `read-fonts` in the lockfile, or comments that tell the truth about why there are two.

### 33. A 40-day stash that no longer applies, and may hold unlanded text work

`stash@{0}: WIP on master: db1352f` (2026-07-19), 19 files, +1287/−164. `git apply --check` fails on 10 of them; its `Cargo.toml` hunk adds `unicode-segmentation` and `unicode-normalization`, both already in the current manifest with near-identical comments — so it is at least partly landed. The question is `paragraph_shaper.rs` (+151) and `text_buffer.rs` (+64). It predates the `ui_runtime` decomposition and will never apply again.

**Effort:** S. **Done means:** somebody reads the two text hunks, and the stash is dropped.

---

## Part 3 — Absent by decision

These are recorded non-goals with reasons. They are listed so nobody re-derives the decision, and in two cases because the *record* is what needs fixing, not the code.

**The plugin tier (§8) is scaffolding, on schedule.** Nine crates, ~1,100 lines, 192 files, 138 of them `.doc` files reading "skeleton stub … not yet implemented", zero tests, zero dependents, not re-exported by the facade. `specs/…/09-roadmap-and-non-goals.md:101` puts the whole platform in Phase D, last. It does not claim a sandbox, which §9.6 singles out as the trap that matters. §8 is 1,297 of the spec drafts' 3,590 lines, so any plan reading "the crates exist" as progress will be wrong by two orders of magnitude. Two crate-level docs do over-claim in present tense and should be fixed (Part 4).

**Phase C surfaces are absent wholesale**: §4.11 infinite workspaces, §5.4 overlay host, §5.5 foreign surfaces, §5.6 offscreen output — `InfiniteWorkspace`, `WorldCoord`, `OverlayHost`, `ForeignSurface` all return nothing. Nothing for the two stated targets. One caveat: §5.6 is nearly free — the CPU backend already rasterises a scene into a buffer (that is how the PNG showcase examples work), so a typed offscreen API is mostly naming the inputs the spec names.

**Layer-shell keyboard delivery, xdg-popup and wlr v5 `set_exclusive_edge`.** All genuinely absent (`crates/adapters/obscur-shell-layer/src/surface/layer_state.rs:294-309` binds only Pointer and Touch; `LayerInput` has no key variant; zero `popup` hits; SCTK 0.19.2 binds wlr-layer-shell 1..=4). All three are stated boundaries — `crates/adapters/obscur-shell-layer/src/lib.doc/_module.md:28-33`: "a keysym-to-`PhysicalKey` mapping is a table this adapter does not have, and inventing an approximate one would put wrong keys into an application rather than none … it is a boundary, and it is stated here rather than discovered" — and rows 89–91 of `docs/api/12-reality.md`. And no current consumer needs them: `../obscur.virtual-keyboard` deliberately takes no keyboard focus (`keyboard-app/src/main.rs:52-54`), sends keys through its own input-method adapter, and draws palettes in-surface. **But see item 39 in Part 4** — the capability model lies about this, and that part is not a decision.

**macOS and Windows font enumeration.** `kit_bootstrap.doc/system_font_set.md` carries a `# Scope` section: Linux only; Core Text and DirectWrite "want a real platform API rather than a CLI shell-out, and neither is built". The consequence is not invisible text — `kit_bootstrap.rs:346-387` embeds Noto Sans as `BUNDLED_FALLBACK` and registers it under `sans-serif`, pinned by `crates/facade/obscur-kit/tests/text_stack_install.rs:381-428` — but that a macOS or Windows app renders in Noto instead of San Francisco or Segoe UI, **with no diagnostic saying so**. One `report(...)` next to the empty-set branch closes the residue, in a file that already emits that shape of note for a missing appearance source. (The `obscur-system-{macos,windows}` crates are stub *system-service* adapters — six 5-line structs — and are unrelated to fonts.)

**Gamepad, haptics and spatial navigation.** Zero references outside `obscur-interaction`. `docs/api/06-input.md:535-542` records all three, and spatial navigation states its reason: "arrows belong to the editing session, and taking them for navigation needs a rule about when a control is a *grid* that this toolkit does not have." That is a real accessibility gap — arrow keys never move focus, so a radio group or a grid cannot be traversed — with a coherent reason rather than an oversight. Haptics is the nearer of the three, since §6.16 needs it (item 7). `debt-closure.md` records the deletion of gamepad as considered and refused: it is spec-assigned and fillable; what is missing is an adapter, not a producer that cannot exist.

**`obscur-routing` is a library tier the application drives.** §6.13 ends "Application navigation policy remains outside the toolkit" (`specs/obscur-kit-2-spec-drafts/06-platform-windows-and-services.md:231`); `RouteRegistry` resolves to an application-chosen target id the toolkit "stores verbatim and never interprets" (`route_registry.rs:9`, a `Box<str>`). A runtime consumer is not merely absent, it is precluded. It is exercised through the facade by a spec-derived suite (`crates/facade/obscur-kit/tests/spec_ch6_routing.rs`) plus 49 in-crate tests. Two ledgers describe this boundary in the vocabulary of debt and should be corrected (Part 4).

**`@container` fails closed, by architecture.** `with_container` (`crates/domain/obscur-style/src/matching/media_environment.rs:37`) has only test callers; both production sites build `MediaEnvironment::viewport`. The author *is* signalled: `crates/frontend/obscur-css/src/parser/stylesheet_parser.rs:252-266` records `@container` as an unsupported at-rule and `css_pipeline.rs:69-71` reports it as a `css.unsupported_at_rule` warning on the standard `App` path. And it is not a wiring job: `MediaEnvironment` is one global environment for a whole-tree match pass that runs strictly before layout, so real support needs a `container-type` property, per-element container sizes and a layout-interleaved restyle (`stylesheet_parser.rs:259-263`). `docs/conformance-audit.md:273` logs it closed as "fails closed + diagnostic; still not implemented". Two docs and two spec lines contradict this (Part 4).

**Drag preview and drag autoscroll.** `crates/application/obscur-runtime/src/orchestration/drag_coordinator.doc/DragCoordinator.md`, "# Not carried": the preview is not painted and autoscroll is not driven, because "both are scene/scroll concerns, and neither is what commit/cancel is about". `DragAutoscroll` is named nowhere else in the workspace. A drag has no visual and will not scroll a list at its edge — a bounded, stated limit. This is the model of what a documented gap looks like here, and the contrast that makes item 3 look accidental: the same crate's other half stops at a boundary the code names.

**A `.okui` file's structure is materialised once.** `docs/api/09-authoring-formats.md:110-113`: `<If>` and `<For>` are materialised statically by the bridge, so structure is fixed for the life of the process; only values reach the screen through `ScreenData` (`app.rs:322-330` re-lowers per frame). A list that changes length or a panel that appears conditionally needs the `okui!` macro instead. Honestly recorded now — the undocumented half was the finding `../test-app` caught — but the two front ends are still presented as parallel routes at `09:6-14`, and the choice between them is load-bearing.

**`BackpressurePolicy::bounded(n)` does not enforce its cap, and it barely matters.** At capacity with the default `skip_stale: true`, `decide` returns `DropStale` (`crates/application/obscur-runtime/src/scheduler/backpressure_policy.rs:69-77`) and `frame_scheduler.rs:49-54` pushes anyway — `DropStale` names an action the scheduler never performs. But `SchedulerQueue` is a `BTreeSet` keyed by frame reason over 8 `FrameReason` variants, so the pending set is structurally capped at 8 zero-payload entries under every policy, `unbounded()` included; `bounded(n)` overshoots by at most `8 − n`, and for `n >= 8` the branch is unreachable. `bounded(n).with_skip_stale(false)` bounds correctly and is tested. §3.5 never asks for a pending cap. What is genuinely wrong is the *rationale*: `frame_scheduler.doc/request.md` defends the push by saying the queue coalesces by reason — true for the `Coalesce` branch, false for `DropStale`, which fires only when a *different* reason arrives at capacity. Fix the doc, or delete `max_pending`. S.

---

## Part 4 — Rotten: docs, comments and tests that are no longer true

This part matters more than its size suggests: this codebase has zero `TODO`, `FIXME`, `XXX`, `HACK`, `todo!()` and `unimplemented!()` across 1,969 Rust files, and only two `#[ignore]`s, both justified. The comment policy works exactly as intended — and it means the entire debt surface has migrated into 17,930 prose `.md` files that nothing mechanically checks. That is where every item below rotted.

**34. `::before`/`::after` work, and the compiler tells every author they do not.** `runtime/style_phase.rs:189` calls `resolve_generated_content`, which cascades pseudo rules and reads `PropertyId::Content` (`orchestration/generated_content.rs:219`); `property_compiler.rs:278-286` compiles `content`; the spec-derived facade test `crates/facade/obscur-kit/tests/spec_ch4b_generated_parts.rs:31` compiles `.badge::before { content: "!" }` through `StylesheetCompiler` itself and asserts the `!` is painted — verified green today (3 passed). Meanwhile `crates/frontend/obscur-css/src/compiler/selector_compiler.rs:326-336` pushes `UnsupportedFeatureKind::PseudoElement` for *every* pseudo-element under the comment "No `content` property and no generated box" — false on both counts — and four assertions pin it (`selector_compiler.rs:454-464`, `:466-473`, `stylesheet_compiler.rs:578-589`). Those are ordinary in-crate unit tests, not spec-derived; §4.16 (`specs/…/04-layout-css-and-motion.md:339`) says "`::before` and `::after` are supported for decoration only" and asks for no diagnostic, so amending them is safe.

Two traps for whoever fixes it. `docs/api/12-reality.md:85` states the feature is inert and says "Re-confirmed at both ends" — that row is the thing to fix first, since it is the page a reader consults *because* they do not trust the others. And `crates/domain/obscur-style/src/selector/selector_matcher.rs:133-141` carries the same false justification above `SimpleSelector::PseudoElement(_) => false` — the comment is wrong but the behaviour is correct and load-bearing (generated content matches by its own path at `generated_content.rs:110-118`; flipping the matcher would make `.badge::before { color: red }` redden `.badge` itself). Fix the comment there, not the code. The diagnostic should be narrowed to exempt `before`/`after` only; `::first-line`, `::selection` and `::marker` genuinely are unsupported. **S.**

**35. `docs/visual-remaining.md` declares four shipped features unbuilt, and ranks the top one as next to build.** Transformed hit-testing exists (`runtime/hit_testing.rs:23-24` `inverse_transforms`, `:72` `accumulated_transform`); the colour-matrix filter family exists (`orchestration/filter_color_matrix.rs` plus a real wgpu `ColorMatrix` pass at `render-vello/src/renderer/segmented_compositor.rs:118`); `mix-blend-mode` is read at `orchestration/composite_effects.rs:56`; percentage translate is now refused with a diagnostic rather than silently zeroed. Last touched 8 July. A reader planning work from this file would re-implement three features and skip the one genuinely still absent — `clip-path`, confirmed at `vello_renderer.rs:520` (`SceneEffect::Mask => SupportLevel::Unavailable`). **S.**

**36. `docs/api/12-reality.md` — the page whose whole thesis is "do not trust this without re-running its grep" — is wrong in four rows.** (a) The Diagnostics row says `set_diagnostic_sink` has zero facade call sites; `app/app_parts.rs:136-147` installs a chosen sink or `StderrDiagnostics` under `#[cfg(debug_assertions)]` on every boot, citing audit 3-C, closed 2026-08-08 — the page was re-measured 2026-08-04. What survives is narrower and worth keeping: a *release* build with no sink chosen still gets `NullDiagnosticSink` (`ui_runtime/construction.rs:86`). (b) The `ch` row names four production supply sites for `ChMetrics`, and emphasises the count; one of them is `frame/frame_pipeline.rs:107`, inside a pipeline nothing runs — there are three. An unwired duplicate corrupted the audit that exists to find unwired duplicates. (c) The font row (item in Part 3) now over-claims a defect: it says text has no source off Linux, while a test proves the bundled fallback paints. (d) The `::before` row above. Also: the MARKUP_ONLY_ELEMENTS citation quotes `&["a", "fieldset"]` where `tests/ua_stylesheet_vocabulary.rs:66` has `&["a", "canvas", "fieldset"]` (the verdict is right, only the quoted line rotted) — worth one reader's attention on its own, because `a` having no widget and no primitive means **the toolkit ships no hyperlink of any kind**. And the page has no row at all for the layout observers (item 24) or for the two dead islands in the delete section. **S** to correct; do it in one pass at the moment the uncommitted tree lands, not before.

**37. Four public rustdoc files talk an author out of features that work.** `crates/domain/obscur-view/src/view/event_name.doc/EVENT_INPUT.md:3` and `EVENT_CHANGE.md:3` both say "Reserved now for later wiring; the runtime does not yet fire it" — contradicted by dispatch at `ui_runtime/text_edit_commands.rs:109`, `ui_runtime/text_editing.rs:112`, `ui_runtime/slider.rs:99`, and by `crates/application/obscur-runtime/tests/text_input_events.rs:73,281`. This is what an author reads on docs.rs for the two event names every form needs. `layout_keyword.doc/OVERFLOW_SCROLL.md:1` and `OVERFLOW_AUTO.md:1` say "no scrollbar is synthesised yet" — one is painted, hit-listed for DevTools and pinned (`orchestration/scroll_container_paint.rs`, `runtime/tree_inspector.rs:159`). `layout_keyword.rs:194-197`'s block comment says every visible border style folds to `solid`, contradicted three lines down by its own `BORDER_STYLE_DASHED.md` and by `border_paint.rs:184`. `docs/api/06-input.md:543-546` still says `FocusRestore` has no runtime caller — wired today by commit `d616f5fe` (`runtime/handler_registry.rs:28,209,213-214`); the `FocusTree` half of that row is still true, so the row needs splitting, not deleting. Highest cost-to-effort ratio in the ledger: a doc that under-promises is never falsified by a bug report. **S.**

**38. `docs/layer-consumer-audit.md`'s finding 2-K is diagnosed wrong and has no status row.** Both halves as written are refuted: open style scope works and is pinned by `crates/facade/obscur-kit/tests/component_output_is_styled.rs` (5/5 green, including `a_descendant_selector_climbs_from_the_document_into_an_open_component`), and inline `style="…"` reaches a component internal fine (`orchestration/inline_styles.rs:88` walks every node with no scope gate; measured landing on the span inside a button). The real defect underneath is item 1, and it is larger. The consumer's own CSS still asserts the inline-style claim (`suggestion_bar.css:98-101`), so the `::part()` workaround it drove people to was chosen partly on a false premise. Two of the audit's six named elements (`.setting-hint`, `.drag-grip`) sit inside plain `<div>`s and should be styled by items 1's mechanism — that half looks like generalisation from the button cases. **S** to rewrite the finding; the fix is item 1.

**39. `LayerWindow::supports` reports `ExclusiveKeyboard` as `Native` while delivering no keys.** `crates/adapters/obscur-shell-layer/src/surface/layer_window.rs:347-361` — the comment says "wlr-layer-shell names all three keyboard modes", true of the protocol, false of this adapter. The feature's own doc states the intended contract: "Where it is unavailable the honest fallback is on-demand focus, reported as `Emulated`" (`crates/domain/obscur-windowing/src/port/layer_backend.doc/ExclusiveKeyboard.md`). So a host that does the right thing — asks `supports()` before choosing a focus policy — is told the capability is native, hands the compositor an exclusive keyboard grab (`layer_window.rs:384`) and then eats every keystroke, which on a wlroots desktop means the user cannot type anywhere until the process is killed. Nothing reaches that state by default (`LayerFocusPolicy::default() == Never`), and `grep ExclusiveKeyboard docs/` finds only a passing mention, so no ledger tracks it. The capability model exists precisely so a host is not lied to. **S** — `Limited` or `Emulated`, plus a sentence in `supports.md`.

**40. Five stale ledger rows, each cheap, each capable of sending someone to re-do closed work.**
- `media_environment.doc/MediaEnvironment.md` and `docs/conformance-audit.md:542` both say the `unsupported` field has no production reader — false since `css_pipeline.rs:69-71`.
- `specs/obscur-kit-2-spec-drafts/04-layout-css-and-motion.md:260` and `specs/obscur-kit-2-crate-architecture/02-crate-catalog-and-dependencies.md:397` list container queries as **supported**. The spec lying to a reader is worse than the feature being absent; a one-word edit each.
- `docs/grid-remaining.md` (6 July, predates the §11 track-sizing rewrite) needs two rows struck — `justify-content`/`align-content: stretch` is implemented (`grid_layout.rs:228-231,391-394` → `grid_track_sizing.rs:595-597,523-541`) and `fr` in an implicit track flexes (both verified by running the algorithm) — and three corrections: baseline falls back to `stretch` not `start`; `grid-column: main 2` drops the whole shorthand rather than resolving the first occurrence; and its stated reason for the stretch row ("`JustifyContent` has no `Stretch` variant") was false even when written (`crates/domain/obscur-layout/src/model/justify_content.rs:19-20`).
- `docs/servo-algorithm-catalog.md` rows 17–19 and 23: B1 is closed (`crates/foundation/obscur-geometry/src/primitive/corner_radii.rs:157-177` applies the single §5.5 scale, called at `scene_coordinator.rs:653-664`), and B2 is closed as written — a positioned `z: auto`/`0` box now sorts above non-positioned siblings via `PaintLayer` (`stacking.rs:130-155`). Its two variants are not a partial §9.9 but a documented encoding: layers 2 and 7 are carried by the primary z key, and 1/3/4/5 are deliberately collapsed because `float` is not a computed property here and splitting block from inline would reorder non-positioned siblings out of document order (`stacking.doc/PaintLayer.md`).
- `docs/api/01-entry-points.md`'s `App` table omits `font`, `font_face` and `diagnostics` (`app.rs:151,161,261`) and its `AppError` list names five of nine variants. Drift in the safe direction, but it hides two channels: `../obscur.virtual-keyboard/crates/keyboard-app/src/main.rs:765` still installs its sink through `with_runtime` with a long comment about ordering, for a job `App::diagnostics` has done correctly since audit 3-C — a consumer using the escape hatch because the page it reads does not list the builder. And `docs/api/02-view-model.md:145-147` names `on_scope_command` as reaching `App`; it does not.
**S** for all five.

**41. `CLAUDE.md`'s calibration numbers are stale — the file every agent reads first.** It claims "168 integration-test binaries (139 in obscur-kit alone)"; today it is 178 workspace-wide and 146 in the facade. It claims "6631 passing as of 2026-08-04"; `grep -rnE '^\s*#\[test\]' crates/ xtask/` counts **6,980** attributes and `docs/perf-audit.md:23` measured 6,950 on 2026-08-12. It claims a 360-warning clippy baseline; today is 366. The test-binary count is the stated cause of the 8.2 GB peak and has grown 6 % since it was written, so the memory ceiling in that table is now optimistic on a box where an OOM kill takes down every terminal at once.

Note what is *not* wrong here: the other three test counts in `docs/` are correctly dated records of closed campaigns and must be left alone — `debt-closure.md:25`'s 6,439 is load-bearing arithmetic (`6,481 − 43 + 1`) proving no surviving test was removed. The clippy figures do not disagree either; `perf-audit.md:24` states its 359 against an explicit "359–360" band. The one genuine contradiction is that `CLAUDE.md:63` (6,631) and `docs/api/12-reality.md:16` (6,439) claim the **same measurement date** and differ by 192. **S** — re-measure once, fix `CLAUDE.md` and that date, touch nothing else.

**42. Three uncommitted campaigns did not update `docs/api/`.** `docs/api/04-styling.md:98-99` lists the supported property set with `overflow` and no `overflow-x`/`overflow-y`, while the uncommitted campaign adds both across 20 files. `grep -rln ThemeVars docs/` matches no API page — `obscur_kit::ThemeVars` and `KitBootstrap::install_system_theme` are undocumented outside their own `.doc` files. `docs/api/README.md`'s contract is "an unmarked API is wired", so a page that omits a shipped property lies by omission. The layer campaign *did* update its pages, which is the standard to hold the others to. **S**, and cheapest while the campaign is still in hand — once committed, the omission reads as a decision.

**43. Small test hygiene.** Five tests self-skip and report green when fontconfig is missing (`crates/facade/obscur-kit/tests/system_font_set.rs:28,32,61,66,115`, `named_font_family.rs:142,150,155,162,194,199` — each `eprintln!("skipped: …"); return;`), which is exactly the trap `Cargo.toml:26-34` explains was closed at the feature level and left open at runtime; cargo swallows the stderr without `--nocapture`, and no CI step installs or asserts fontconfig. `crates/facade/obscur-kit/tests/okui_screen_state.rs:252` (`big_lowering < bridge * 4`) runs at a measured 2.8–3.1× against its 4× ceiling — thin, though I could not make it fail in 38 runs including under 4× CPU contention; its companion at `:257` has 5.5× headroom and is not at risk. The suite's more hardware-sensitive assertion is `crates/facade/obscur-kit/tests/spec_ch5b_overlay_host.rs:81`, the only absolute wall-clock budget (`< 5 ms`), on a CI runner far slower than this box. Two spec-chapter oracles assert what the type system already guarantees (`spec_ch7_accessibility.rs:56`, `spec_ch7_goldens.rs:110` compare a rasterized and an un-rasterized runtime, which cannot diverge because `Renderer::render` takes an owned snapshot by shared reference). Two literal tautologies exist in the whole 7,000-test suite (`obscur-devtools-protocol/src/message/topic.rs:23`, `inspectable.rs:39`). Six `App` API doc examples are ```ignore-fenced and do not compile as written (`app.doc/{painter,painter_id,on_command,layer,displays,diagnostics}.md` — missing `use` lines, a `?` in a `fn main()` with no `Result`). Five orphan `.doc/*.md` files exist in the untracked appearance providers — out of 17,930, which is the real result: the doc discipline is holding. And `Cargo.toml:20` still reads `repository = "https://example.invalid/obscur-kit"` while a real origin exists; harmless (every crate is `publish = false`) except that it lands in rustdoc source links. **S**, all of it.

---

## What was dropped

Eighteen claims that the sweep carried, and that the repository's own ledgers state as open, do not survive contact with the tree:

1. `docking_layout.rs:174` as an instance of the flex clamp — the clamp is structurally inert.
2. `custom_layout.rs:69` likewise — inline axis cannot bite, no runtime consumer.
3. "`@container` gives the author no signal" — a warning is reported on the standard `App` path.
4. "`@container` is one call away from working" — it needs a `container-type` property and a layout-interleaved restyle.
5. "`obscur-routing` is unwired debt" — a §6.13 boundary, exercised by a facade spec suite.
6. Grid `justify-content`/`align-content: stretch` unimplemented — implemented; verified by running the algorithm.
7. Grid `fr` in an implicit track collapses to its floor — it flexes; same verification.
8. Grid `align-items: baseline` "silently falls back to `start`" — it falls back to `stretch`, which is the CSS initial.
9. Grid `grid-column: main 2` "ignores the integer" — the whole shorthand is dropped as invalid.
10. Servo catalog B1 (corner-radius scaling) — closed.
11. Servo catalog B2 (paint layers) — closed as written; what remains is a documented encoding.
12. Layer audit 2-K, "an open component's internals take no rule of any kind" — refuted by a passing test and by probe.
13. Layer audit 2-K's second half, "inline `style` is dropped on the same elements" — refuted by probe.
14. "macOS and Windows get an empty font set, so text is invisible" — a bundled Noto Sans is registered and painting is pinned by a test.
15. "Layer-shell keyboard/popups/v5 are a live requirement of `../obscur.virtual-keyboard`" — that consumer wants none of the three, in writing.
16. "`initial`/`unset` silently do nothing on ~25 keyword properties" — the declaration wins the cascade and every consumer's fallback is the CSS initial.
17. "IME is entirely unwired" — the full composition lifecycle is mapped in `app/runner.rs`.
18. "`BackpressurePolicy::bounded` yields an unbounded queue" — the queue is structurally capped at 8 entries under every policy.

Two more were refuted by the sweep agents before they reported: SVG group transforms are *not* dropped (usvg pre-resolves them into absolute path data), and `okui_screen_state.rs:257` is not a flake risk.

Fourteen further claims stood but needed material correction — most usefully items 1, 5, 9, 10, 17 and 18 above, where the recorded cause was wrong, the scope was wrong, or the defect turned out to be bigger than its ledger row.

The number itself is the finding. Of twenty-four load-bearing claims put to an adversary, eleven stood — seven of those only after correction — and thirteen were overstated. **No claim collapsed to literally nothing; every one left a residue.** But six were re-diagnosed into a *different* defect, and in two cases a larger one: 2-K's open-scope story became the `<button>` scope defect that heads this ledger, and the overflow-clip row became four sites and a visible corner artefact. The repository's ledgers are unusually honest and unusually stale at the same time. That combination is what item 31 exists to fix — this sweep found two large dead islands (`obscur-tree/src/transaction/`, the `obscur-motion` driver tier) and one whole spec chapter's worth of unwired code (the layout observers) that *no* ledger names, because a grep can only confirm a symbol you already suspect.

---

## The three things I would do first

**1. Push.** Item 0. One minute, and it is the only unbounded cost in the document. Everything else on this list is an argument about priorities; this one is an argument with a disk.

**2. Fix the `<button>` style scope.** Item 1. It is silent, it hits plain markup rather than an advanced feature, it applies to every shipped control, and it has been misdiagnosed once already — so the next person to look at it will also spend a day. It costs an application author real time in the most ordinary situation there is (a label inside a button), and the failure is invisible because the element still computes *a* style. The fix is a decision (slotted content keeps its author's scope) plus a test per built-in control.

**3. Open the `App` doors for text input, component registration and accessibility.** Items 2, 8 and the first bullet of 22. These are three small builder methods on a type that already exists, and together they move `App` from "the recommended entry point that cannot build a form, cannot register a component, and is invisible to a screen reader" to something an author can finish an application with. Every one of the four consumers on disk, in and out of the workspace, opens its `main` with a `with_runtime` loop working around them.

Honourable mention, because it is one line: **item 10**, `position: sticky` in `is_positioned`. Three tokens in `stacking.rs:160-162` fixes both a wrong paint and a wrong click on the single most common sticky pattern there is.

## The one thing I would delete

**`crates/domain/obscur-tree/src/transaction/`** — 990 lines across seven files (`tree_transaction.rs`, `tree_change.rs`, `tree_change_set.rs`, `tree_commit.rs`, `staged_ref.rs`, `tx_error.rs`, `mod.rs`). `rg -w 'TreeTransaction|TreeChangeSet|TreeCommit|StagedRef|TxError|TreeChange' crates tools` returns zero hits outside `obscur-tree/src`, and zero hits inside `obscur-tree/src` outside `transaction/` itself. Not one other crate, not one test, not the rest of its own crate.

It is a second spelling of §3.2's commit boundary, and the first spelling won: `crates/application/obscur-runtime/src/transaction/` (`UiTransaction`, `UiOperation`, `TransactionCommit`, `ReconcileEnactor`, `ProvisionalTransaction`) is genuinely driven. So a reader hunting for "where does a tree change get committed" finds the wrong one, and 990 lines of tested-but-unreachable code taxes every future refactor of `RetainedTree`. The house policy is to delete unreachable code rather than keep it against a future fix, and nothing here is held alive by a spec oracle — unlike the layout observers, two of which a spec test constructs and which therefore cannot simply go.

Delete it, or, if the owner wants it kept, say in its `_module.md` that the runtime's transaction module supersedes it — so the next sweep does not spend an afternoon rediscovering that nothing calls it.

Two runners-up, both smaller and both genuine removals rather than deferrals: `FramePipeline` out of the prelude (item 26 — a second, componentless, diagnostics-free frame pipeline that is the first one a user meets, and that has already corrupted one ledger row), and `RenderExtension`/`CustomPass` (item 26 — either wire the extension list through `Renderer::render` or say custom passes are a non-goal; today a host gets a compile, a green test, and no pixels).
---

## Appendix — six items the sweep could not see

These came out of the 2026-08-28 audit-closure session itself, in the reports of
the agents doing the work rather than from anything on disk. A sweep of the
repository cannot find them, because the repository does not record them
anywhere. Each was re-checked against the tree on 2026-08-28 before being
written down.

**A. Two live pieces of reasoning in the layout code contradict each other, and
one of them is now load-bearing.** `orchestration/layout_coordinator.rs:276-280`
argues, at length and with a spec citation, that **a tight constraint alone
cannot carry definiteness**: `min == max` is also what an arranging caller sends
a box whose height it measured *from that box's content*, so reading
definiteness off the range "would call a content-derived height specified — the
exact conflation `BoxConstraints::percent_basis` exists to end." The fix for
audit 2-I then added a branch to `algorithm/flow_layout.rs`'s
`definite_content_height` that does precisely that: it derives a definite
containing block from a tight block axis. Both are in the tree, both are
reachable, and the suite is green either way. One of the two is wrong, and
nothing will tell you which — this is the kind of disagreement that surfaces
years later as an unreproducible percentage bug. **Effort:** M, and it is a
decision before it is code. **Done means:** one of the two comments is deleted
because its claim was withdrawn, and the surviving rule is stated in one place
that both call sites reference.

**B. Two more sites assign a size without consulting the box's own
min/max — the same defect that was just fixed in flow.**
`algorithm/grid_layout.rs:501-506` hands a both-axes-`Stretch` item
`(inner_w, inner_h)` straight from the cell, and `model/inset.rs`'s
`resolve_absolute_box` does the same for a box pinned by both block insets.
css-align clamps a stretch size by the box's min/max; CSS 2.1 §10.7 re-runs the
inset resolution against `max-height`. Nothing tests either. **Effort:** S each.
**Done means:** a `max-height` box stretched in a grid cell, and one pinned by
`top` + `bottom`, each report the capped size.

**C. `lower_custom_value` still mints open-set keywords into a throwaway
interner.** `frontend/obscur-css/src/compiler/custom_value.rs:39` builds a
`CssSymbolInterner` that is dropped when the function returns, and its own
comment says so. For a property with a `layout_keyword_mapper` this is now
harmless — an unknown word is refused outright — but a property with an *open*
keyword set (`gap: normal`, `letter-spacing: normal`, `visibility: hidden`,
`painter: none`) still receives a symbol nothing can resolve back to a name.
Inert today because nothing reads those symbols for behaviour; a live bug the
moment something does. **Effort:** S. **Done means:** either the open-set
properties resolve through a shared interner, or carrying a keyword for them is
refused the way the closed sets are.

**D. `InternedSymbol::named` and `from_index` are not partitioned from each
other.** `foundation/obscur-identity/src/key/interned_symbol.rs:31,37` — both
set the top bit, so a sheet-local index can in principle equal the FNV hash of
some name (~n/2³¹ for n interned words). The 2026-08-28 fix made sheet-local
symbols disjoint from the *reserved* range, which is the collision that was
measured; this one is the residue. No live path compares the two namespaces —
keyword values, custom-property names and command names never meet in one map —
so it is a hazard rather than a defect, and `interned_symbol.doc/index.md` says
so. **Effort:** S. **Done means:** the two producers occupy provably disjoint
ranges, pinned by a test, so the argument does not have to be re-made.

**E. The `TurnClock` tests cannot fail.** `facade/obscur-kit/src/app/turn_clock.rs`
is the fix for audit 2-L's motion clock, and all four of its tests exercise
`TurnClock` itself — a type the fix introduces, so without it they do not
compile rather than fail. The defect lived inline in `layer_runner.rs`'s loop,
which needs a Wayland compositor to run. The change is correct by reading and by
symmetry with the windowed runner, and it is the one item in this document
verified that way. **Effort:** M — it needs a harness that can drive the layer
loop without a compositor, which nothing has. **Done means:** a test that fails
against the pre-fix arithmetic, or a written statement that this path is
owner-verified only, next to the other two paths already marked that way.

**F. `checks/layer-consumer` still contains no `var(`.** The window profile
gained one on 2026-08-28, and it earned its keep immediately: ablated, it
catches the class of defect that had escaped the whole suite. The desktop-surface
profile — the one that matches the project's stated on-screen-keyboard target —
has no equivalent. **Effort:** S. **Done means:** the layer profile measures a
computed value that came through a custom property, and fails when the
substitution parser is not installed.


---

## Pass 2 — what running it found

Five lanes executed the toolkit instead of reading it: every runnable example, both renderers against a byte-identical command stream, the vendored W3C text as an oracle, degenerate input through the public surface, and malformed input through the four parsers. Adversaries then re-ran the load-bearing claims from their own probe crates. What follows is what survived that, ordered by what it costs to leave alone. Every item carries the command and the verbatim wrong output, because that is what a static sweep could not produce.

All probes were built out of workspace on the `checks/facade-consumer` pattern (empty `[workspace]`, one path dep on the facade), through the memory-capped `systemd-run` wrapper. Nothing in the repository was modified. Note for anyone re-running these: `/tmp` is a 16 G tmpfs that sat at 80% during this pass and killed three builds with `Disk quota exceeded` / `ld terminated with signal 7 [Bus error]` — set `CARGO_TARGET_DIR` and `TMPDIR` outside it.

---

### Part A — breaks an application

#### A1. Every math function containing a percentage is silently discarded

`width: calc(100% - 40px)` becomes `auto`. `width: calc(50%)` — a math function with a percentage and no arithmetic at all — measures 0 where the identical `width: 50%` measures 200.

```
.a{flex:none;height:20px;width:50%}                     [control: bare %]
    [0 css diagnostic(s) raised for this sheet]
    got  x=0.00 y=0.00 w=200.00 h=20.00   want w=200
.a{flex:none;height:20px;width:calc(100px + 20px)}      [control: calc, no %]
    got  x=0.00 y=0.00 w=120.00 h=20.00   want w=120
.a{flex:none;height:20px;width:calc(100% - 40px)}
    got  x=0.00 y=0.00 w=  0.00 h=20.00   want w=360
.a{flex:none;height:20px;width:min(50%, 100px)}
    got  x=0.00 y=0.00 w=  0.00 h=20.00   want w=100
.a{flex:none;height:20px;width:clamp(100px, 50%, 150px)}
    got  x=0.00 y=0.00 w=  0.00 h=20.00   want w=150
.a{flex:none;height:20px;width:calc(50%)}
    got  x=0.00 y=0.00 w=  0.00 h=20.00   want w=200
.a{flex:none;width:100px;height:calc(100% - 40px)}
    got  x=0.00 y=0.00 w=100.00 h=300.00  want h=260
.o{position:absolute;left:calc(50% - 50px);top:20px;width:100px;height:50px}
    got  x=0.00 y=20.00 w=100.00 h=50.00  want x=150
.o{position:absolute;left:50%;...}                      [control]
    got  x=200.00 y=20.00 w=100.00 h=50.00
.a{margin-left:calc(25% + 10px)}   got x=  0.00   want x=110
.a{margin-left:calc(10px + 10px)}  got x= 20.00   [control, correct]
```

Every sheet raised **zero** diagnostics (checked `.errors` *and* `.unsupported`). The value survives compilation intact — `Declaration { property: Width, value: Calc(Diff(Leaf(Percent(100.0)), Leaf(Px(LogicalPx(40.0))))) }` — and dies at the layout bridge.

`crates/application/obscur-runtime/src/orchestration/length_resolution.rs:83` — `Some(PropertyValue::Calc(expr)) => expr.resolve(ctx).map_or(Dimension::Auto, Dimension::Px)`. `LengthContext` carries no percentage basis, so `LengthExpr::resolve` returns `None` for anything containing `%` and the declaration collapses. The pixel-only twin is `:111` (`.unwrap_or(0.0)`). `grep -rn "PropertyValue::Calc" crates/application/` returns exactly those two lines. Structural cause: `crates/domain/obscur-layout/src/model/dimension.rs:5-18` is `Auto | Px | Percent | MinContent | MaxContent | FitContent` — no expression variant, so nothing symbolic can survive to the point where the containing block is known.

The runtime's own doc directory records the behaviour (`length_resolution.doc/length_context.md`: "a `%` term inside `calc()` therefore does not fold here (that value falls back to `auto`/`0`, still without an unsupported-feature diagnostic)"; `margin_edges_of.md` likewise) — a known wart, not a stated non-goal. The author-facing contradiction is `docs/api/04-styling.md:229-233`, which tells authors math functions "are accepted wherever a `<length>` is" with `ch` as the sole caveat.

**Effort:** L — it needs a `Dimension` variant that carries the expression to layout, or a percentage basis threaded into `LengthContext`. **Done means:** `width: calc(100% - 40px)` in a 400px CB measures 360, `min()`/`max()`/`clamp()` with a percentage resolve, and the two `length_resolution.doc` files plus `docs/api/04-styling.md` stop disagreeing.

#### A2. An unrecognised value does not get dropped — it overwrites the valid declaration before it

```
$ /home/alex/probe-lane-css/target/debug/inv          (parent widened to 137 so a wrong number cannot be a coincidence)
baseline                 Some((60.0, 20.0))    <= .a{width:137px} .b{width:60px;height:20px}
-webkit-fill-available   Some((137.0, 20.0))   <= .b{width:60px; width:-webkit-fill-available; height:20px}
stretch                  Some((137.0, 20.0))
-moz-available           Some((137.0, 20.0))
aut0 (typo)              Some((137.0, 20.0))
red (wrong type kw)      Some((137.0, 20.0))
two rules                Some((137.0, 20.0))   <= .b{width:60px;...} .b{width:-webkit-fill-available}
lower spec later         Some((60.0, 20.0))    <= correct: specificity still wins
multi-token              Some((137.0, 20.0))   <= width: not a length          -- and NO diagnostic
bad dimension            Some((60.0, 20.0))    <= width: 100qq   correctly dropped
bad function             Some((60.0, 20.0))    <= width: frobnicate(3)  correctly dropped
height invalid           Some((60.0, 0.0))
max-width invalid        Some((60.0, 20.0))    <= 50px lost
min-height invalid       Some((60.0, 20.0))    <= 40px lost
$ ... /inv2
margin-left invalid   (x,y,w,h)=Some((0.0, 0.0, 60.0, 20.0))   <= 23px lost
padding-left invalid  (x,y,w,h)=Some((0.0, 0.0, 60.0, 20.0))   <= 11px lost (content 71 -> 60)
left invalid          (x,y,w,h)=Some((0.0, 7.0, 60.0, 20.0))   <= abspos left 30px -> 0
invalid then valid    (x,y,w,h)=Some((0.0, 0.0, 60.0, 20.0))   <= order, not something else
```

Every `<length>` longhand is in scope. `width: 100px; width: -webkit-fill-available` is the progressive-enhancement idiom; here the fallback is destroyed by the enhancement.

`lower_keyword` (`crates/frontend/obscur-css/src/compiler/value_lowering.rs:51`) never fails — an unknown word is minted as an inert interned keyword — so `lower_for_property` returns `Some`, `property_compiler.rs:120-134` builds a `CompiledDeclaration::Standard`, and `stylesheet_compiler.rs:324-330` pushes it into the block, where it outranks the earlier declaration by source order and computes to the property's initial value. **The fix is already half-written:** `keyword_value_is_foreign(property, &decl.value)` is computed three lines below the push at `:341-345` and used only to record a diagnostic, never to skip the push. And that check matches a one-element slice (`longhand_catalog.rs:265`), so `width: not a length` does the same damage with no diagnostic at all.

Three places in the tree assert the opposite: `stylesheet_compiler.rs:336-338` ("so it is dropped in effect"), `restyle_phase.rs:155-157` ("dropped it and kept the rest (§4.13)"), `style_matching_coordinator.doc/collect_invalid_value_faults.md:7` ("a `width: not-a-length` never reaches the cascade").

**Effort:** S. **Done means:** the foreign-keyword check gates the push instead of only the diagnostic, `lower_value` refuses a multi-component value it cannot lower, and a test asserts `width: 60px; width: -webkit-fill-available` measures 60.

#### A3. Negative lengths are applied where CSS requires non-negative

```
$ /tmp/probe-degen/target/release/pa      (.a 200x100 containing .b 60x20; tuples are (x,y,w,h))
control:                 a=(0,0,200,100)   b=(0,0,60,20)
padding: -10px:          a=(0,0,180,80)    b=(-10,-10,60,20)
padding-left: -10px:     a=(0,0,190,100)   b=(-10,0,60,20)
border: -5px solid red:  a=(0,0,190,90)    b=(-5,-5,60,20)
width: -50px:            a=(0,0,0,100)
max-width: -50px:        a=(0,0,0,100)
margin: -10px:           a=(-10,-10,200,100)   [legal, handled correctly]
$ /tmp/probe-degen/target/release/pb
flex gap:0px    -> 2nd item x=40      flex gap:-10px -> x=30 (items overlap)
flex gap:-1e30px -> x=-1e30
font-size:16px  -> h=21.792           font-size:-10px -> h=0.0 (text disappears)
$ /tmp/probe-negrange/...              (independent probe)
later width:-50px    -> a w=0.0    <= .a{width:200px} ... .a{width:-50px}
padding: 10px -10px  -> a=(0,0,180,120) b=(-10,10,60,20)   <= half a shorthand applied
padding:-1e9px       -> b=(-1000000000.0, -1000000000.0, 60, 20)
```

Two things are wrong at once. An invalid declaration **overrides a valid earlier one** (`width: 200px` then `width: -50px` measures 0), and one invalid component of a shorthand applies the other half. `padding: -10px` is the worst: the parent's box shrinks *and* the child is placed at (-10,-10), outside the box that reports as containing it — geometry a browser cannot produce.

Confirmed at compile level: `Declaration { property: PaddingTop, value: Length(Px(LogicalPx(-10.0))) }`, errors=0, unsupported=0.

Site is `crates/frontend/obscur-css/src/compiler/property_compiler.rs:472-483` — `lower_length_longhand` has no per-property range. (`is_length_longhand` at `longhand_catalog.rs:150-205` is only the unitless-zero gate; it is not the defect, it is the right shape for the fix.) The governing rule is already stated three hundred lines away at `property_compiler.rs:459-468`: "a declaration whose value does not match the property's grammar is dropped (CSS Syntax 3 §9)". The numeric-range half was never written.

Rows measured **inert** and not worth a test: `flex-basis: -10px`, `outline-width: -3px`, `min-width: -50px`, non-flex `gap: -10px`. `line-height: -2` and `line-height: -10px` are handled **correctly** (fall back to `normal`, 21.792).

**Effort:** S. **Done means:** an `is_non_negative_longhand(property)` sibling returns `None` from the length path, `.a{width:200px} .a{width:-50px}` measures 200, and `padding: 10px -10px` applies nothing.

#### A4. Percentage `padding` and percentage `gap` compute to zero, with no diagnostic

```
.cb{width:400px;height:300px} .a{width:200px;height:100px;padding-left:10%}
  [0 css diagnostic(s) raised for this sheet]
    node -> x=0.00 y=0.00 w=200.00 h=100.00     want w=240
.cb{...} .a{width:200px;height:100px;padding-left:40px}     [control]
    node -> x=0.00 y=0.00 w=240.00 h=100.00     correct

.a{padding:10%;height:20px}      -> w=400.00 h= 20.00   want h=100
.a{padding:40px;height:20px}     -> w=400.00 h=100.00   [control]

child of .a{padding-left:10%;padding-top:25%}   -> x= 0.00 y=  0.00   want x=40 y=100
child of .a{padding-left:40px;padding-top:100px}-> x=40.00 y=100.00   [control]

.c{display:flex;width:400px;height:50px;gap:10%} -> 2nd item x=100.00   want x=140
.c{display:flex;width:400px;height:50px;gap:40px}-> 2nd item x=140.00   [control]
```

The percentage survives the whole CSS pipeline — `Declaration { property: PaddingLeft, value: Length(Percent(10.0)) }`, `Gap` likewise, errors=0 unsupported=0 — and is discarded at the layout bridge: `crates/application/obscur-runtime/src/orchestration/length_resolution.rs:108`, `Length::Percent(_) | Length::Auto => 0.0` inside `length_px_of`, reached for padding through `insets_of` (`:143-157`, called at `layout_style_mapping.rs:75,86`) and for gap at `layout_style_mapping.rs:110`.

Structural cause: `LayoutStyle::padding`/`::border` are `obscur_geometry::Insets` (plain f32), while `margin` is a `MarginEdges` of `Margin`, which carries `Percent` and resolves correctly (`model/margin.rs:29`) and got a written "# Why this is not `Dimension`" section. The two halves of the box model were modelled differently. Neither `layout_style.doc/padding.md` nor `gap.md` mentions percentages; `layout_style_mapping.doc/layout_style_from.md:25-27` records the consequence without a reason. Nothing here can even emit a diagnostic — `grep -n "sink\|Diagnostic"` over both files returns nothing.

`border-width` shares `insets_of` but does not accept percentages in CSS, so it is not a live instance. `docs/api/04-styling.md:229` ("Percentages stay symbolic until layout knows the containing block") is affirmatively false for these properties.

**Effort:** M. **Done means:** `padding-left: 10%` of a 400px CB is 40px on all four edges (block axis against the inline size), `gap: 10%` resolves, and `layout_style.doc/padding.md` says so.

#### A5. An absolutely positioned box throws away its explicit `width`/`height` when both insets are set, and never consults margins at all

```
CSS base: .cb{position:relative;width:400px;height:300px}
left:10px right:10px width:100px height:50px
    got  x= 10.00 y=  0.00 w=380.00 h= 50.00     want x=10 w=100
left:0 right:0 width:100px margin-left:auto margin-right:auto
    got  x=  0.00 y=  0.00 w=400.00 h= 50.00     want x=150 w=100
top:10px bottom:10px height:50px (left:0 width:20px)
    got  x=  0.00 y= 10.00 w= 20.00 h=280.00     want y=10 h=50
CONTROL  left:10px right:10px (no width)  -> w=380.00   correct (stretch)
CONTROL  left:10px width:100px (no right) -> w=100.00   correct
MARGIN   left:10px margin-left:20px width:100px  -> x= 10.00   want x=30
MARGIN2  left:10 right:10 margin-left:20 margin-right:30 (no width) -> x=10.00 w=380.00  want x=30 w=330
MAXW     left:0 right:0 max-width:100px -> w=400.00   want w=100
```

css-position-3:1468-1472 — over-constrained, ignore `right` (ltr) and solve for it: x=10, w=100. `:1494-1496` is the block-axis twin ("ignore the value for bottom"). `:1455-1459` is the centred-dialog idiom (`left:0;right:0;width:X;margin-inline:auto` → equal margins, x=150).

`crates/domain/obscur-layout/src/model/inset.rs:144` — `(Some(start), Some(end)) => (cb_origin + start, (cb_extent - start - end).max(0.0))` is the one arm of `resolve_axis` that never reads its `explicit` parameter. Margins are not merely un-consulted when `auto`; they are **absent from the signature**: `arrange_pass.rs:421` calls `resolve_absolute_box(cb, width, height, natural, box_sizing, border_padding)` and `inset.rs:42-49` has no margin parameter to receive one — so even the currently-correct stretch arm is wrong once a margin is present (MARGIN/MARGIN2 above).

`inset.doc/resolve_absolute_box.md` describes the behaviour ("The insets win; the explicit/natural width is ignored") and in the next paragraph grounds the function in "CSS 2.1 §10.3.7" — the section mandating the rule it violates. The only test pinning it is the crate-local `inset.rs:243 both_insets_stretch_to_containing_block`, not a spec oracle. The `MAXW` row confirms ledger Appendix B is real and **downstream of this**: fixing the clamp alone still yields 400, because the size being clamped came from the insets rather than the author. `docs/servo-algorithm-catalog.md:169` row E-7 is adjacent but is written as a margin-distribution refinement and does not say an explicit size is discarded.

**Effort:** M. **Done means:** over-constrained resolution drops `right`/`bottom`, `resolve_absolute_box` takes margins and solves the auto-margin equation, and `left:0;right:0;width:100px;margin-inline:auto` centres.

#### A6. `align-content`'s initial value is `flex-start`, not `stretch`

```
UNAUTHORED   .c{display:flex;flex-wrap:wrap;width:100px;height:300px} .i{flex:none;width:100px;height:50px}
  item0  x=0.00 y=  0.00 w=100.00 h=50.00
  item1  x=0.00 y= 50.00 w=100.00 h=50.00
EXPLICIT align-content:stretch
  item0  x=0.00 y=  0.00
  item1  x=0.00 y=150.00        correct
EXPLICIT align-content:flex-start
  item1  x=0.00 y= 50.00        byte-identical to unauthored
```

`specs/w3c/css-flexbox-1.txt:2959-2960` — `Initial: stretch`. The engine computes it correctly; it never reaches the code path. `layout_style_mapping.rs:261-264` delegates `align_content_of` to `justify_content_keyword`, whose no-keyword arm is `FlexStart` (`:252`); default is `layout_style.rs:117`; the gate that skips the whole §8.4 block is `flex_layout.rs:450-451`, `!matches!(style.align_content, JustifyContent::FlexStart)`.

`crates/domain/obscur-style/src/computed/initial_value.rs:205-224` justifies every deferred keyword initial by asserting the consumer's fallback *equals* the CSS initial; for `AlignContent` that is false. `docs/unfinished.md:240` lists ten properties verified that way — `align-content` is absent from the list and is its counterexample. `layout_style.doc/align_content.md` says the field is "Ignored by flex", which the explicit run disproves. `tests/spec_ch4_flex.rs:204-224` passes only because it spells the keyword out.

**Do not fix this by flipping the default to `Stretch`.** `FlexStart` is doubling as the unauthored/`normal` sentinel: `grid_layout.rs:228-231` reads `matches!(style.justify_content, FlexStart | Stretch)` as "normal behaves as stretch", and probing grid confirms that reading is currently producing *right* answers (`display:grid;grid-template-rows:50px 50px;height:300px` correctly packs at y=0/50, because `normal` stretches only auto-max tracks). A one-token flip fixes flex and makes an authored `align-content: flex-start` on a grid indistinguishable from normal.

**Effort:** M. **Done means:** a distinct `Normal` variant exists, an unauthored wrapping flex container puts its second line at y=150, and an authored `flex-start` on a grid is still distinguishable.

#### A7. Negative free space is dropped by three of the four content-distribution sites

```
# one 150x200 item in a 100x100 flex row; free = -50 main, -100 cross
justify-content:flex-start    -> [(0.0, 0.0, 150.0, 200.0)]
justify-content:center        -> [(0.0, 0.0, 150.0, 200.0)]    want x=-25
justify-content:flex-end      -> [(0.0, 0.0, 150.0, 200.0)]    want x=-50
justify-content:space-between -> [(0.0, 0.0, ...)]             correct (safe flex-start)
align-items:center            -> [(0.0, -50.0, ...)]           CONTROL: correct
align-items:flex-end          -> [(0.0, -100.0, ...)]          CONTROL: correct

# two 80x40 items in a 100-wide row; free = -60 (rules out "single item")
justify-content:center        -> [(0.0,..), (80.0,..)]         want x=-30, 50
justify-content:flex-end      -> [(0.0,..), (80.0,..)]         want x=-60, 20

# three 60px columns in a 100px grid area; leftover = -80
GRID justify-content:center   -> [(0.0,..), (60.0,..), (120.0,..)]   want x=-40, 20, 80
GRID justify-content:end      -> [(0.0,..), (60.0,..), (120.0,..)]   want x=-80, -20, 40

# three 100x60 lines in a 100-tall wrap box; free = -80
align-content:center          -> [y=0.0, y=60.0, y=120.0]      want y=-40, 20, 80
align-content:flex-end        -> [y=0.0, y=60.0, y=120.0]      want y=-80, -20, 40
```

css-flexbox-1:2637-2643 for `center`: "(If the leftover free-space is negative, the flex items will overflow equally in both directions.)"; `:3005-3006` the same for `align-content: center`. Only `space-between`/`space-around`/`space-evenly` have a negative fallback (`:2653`, `:2674`) — and those the engine gets right, in both the one-item and two-item runs, so this is not a consistent "safe alignment" policy leaking; it is a fallback written for three keywords applied to all five.

Three sites, all the same shape, all short-circuiting *before* the keyword `match`: `flex_line_placement.rs:281` (`if free_for_justify <= 0.0 || count == 0`), `flex_layout.rs:504` (`if free <= 0.0 || n == 0`), `grid_track_positions.rs:23` (`if n == 0 || leftover <= 0.0`). The correct behaviour already exists three files over in the per-item cross axis, which is the control above — so the two axes of one flex container disagree. `grid_track_positions.doc/content_distribution.md` documents the behaviour without a spec citation or a reason. `tests/spec_ch4_alignment_gap.rs:56-73` asserts `>= 0.0` but only for `space-between`, which stays correct under a fix.

**Effort:** S. **Done means:** `center`/`flex-end`/`end` overflow the line at all three sites, the `space-*` fallbacks are unchanged, and the guard moves inside the keyword `match`.

#### A8. The `column-gap` and `row-gap` longhands do not exist

```
--- flex row, column-gap:20px
    [css-unsupported] UnsupportedCssFeature { kind: Property, name: "column-gap", span: SourceSpan { start: 40, end: 55 } }
    2nd item x = 100.00   want 120   => WRONG
--- flex row, gap:20px (control)          2nd item x = 120.00   => OK
--- flex column, row-gap:20px             2nd item y =  50.00   want 70  => WRONG
--- flex column, gap:20px (control)       2nd item y =  70.00   => OK
--- grid column-gap:20px                  2nd cell x = 100.00   want 120
```

The layout is right and only the property name is unrecognised. Mildest of this group because the compiler is honest: it raises `UnsupportedCssFeature` and `css_pipeline.rs:66-71` forwards it as a diagnostic on the standard `App` path. It is here because `column-gap` is what authored stylesheets actually write.

`crates/frontend/obscur-css/src/compiler/longhand_catalog.rs` has no entry for either longhand; `crates/domain/obscur-style/src/property/property_id.rs:197-198,393` carries a single `Gap`. The fix is smaller than it looks: only the *flex* model is single-scalar (`layout_style.rs:67-68`). `GridLayout` already has independent `column_gap`/`row_gap` fields and `with_column_gap`/`with_row_gap` builders (`grid_layout.rs:24-27,52-62`); they are collapsed by one caller, `formatting_context.rs:22`, `GridLayout::with_template(template.clone(), gap, gap)`.

The only existing acknowledgement is a translation note inside two spec-test module docs (`tests/spec_ch4_alignment_gap.doc/_module.md:10-11`, `tests/spec_ch4_grid.doc/_module.md:11-12`): the spec-deriving agent wrote `column-gap` and the harness had to re-author it as `gap`. That is a workaround, not a non-goal.

**Effort:** S. **Done means:** two catalog entries, two `PropertyId`s, a second field on `LayoutStyle`, and `formatting_context.rs:22` stops passing `gap` twice.

#### A9. One NaN wheel notch permanently freezes a scroll container and makes everything inside it unhittable

```
$ /tmp/probe-degen/target/release/p4
[ok ] hit at (10,10) unscrolled: Some("row0")
[ok ] hit at (10,10) after scroll_by(90): (90.0, Some("row3"))
[ok ] hit after handle_scroll(NaN) then scroll_by(90): (NaN, NaN, None)
[ok ] re-render after NaN scroll: (NaN, NaN, None)

$ (independent probe, adds a sibling outside the port and a recovery attempt)
A2 scroll_by(NaN)      ret=NaN then scroll_by(90)=NaN top=NaN hit(10,10)=(true, None) hit_outside=Some("OUTSIDE")
A3 handle_scroll(NaN)  top=NaN hit(10,10)=(true, None)
A4 re-render x2        top=NaN hit(10,10)=(true, None)
A5 recover set(0.0)    top=0 hit(10,10)=Some("row0") scroll_by(90)=90
A6 set_scroll_top(NaN) top=45 hit(10,10)=Some("row1")     <- the sibling writer refuses it
A7 inf/-inf            +inf->500  -inf->0                  <- only NaN slips through f32::clamp
A8 poisoned boxes      1/47 nodes have non-finite origin.y (the scrollbar-thumb, into inspect_snapshot)
B2 hit target after NaN: classes=Some(["port"]) depth=Some(2)
```

`ScrollOffsets::shift` (`crates/application/obscur-runtime/src/runtime/scroll_offsets.rs:66-71`) is missing the `is_finite` guard its sibling `set` has at `:56-63` and documents at `scroll_offsets.doc/set.md`: "A non-finite `px` is ignored: nothing is written and nothing is refolded, so a `NaN` cannot reach either the map or an observer's coordinates." `shift.md` has neither the guard nor a sentence saying it does not need one. The NaN passes `f32::clamp` unchanged and `refold` then does `layout_box.bounds.origin.y -= NaN` (`:110`) for every node under the container. Entered from `ui_runtime/scrolling.rs:90` (`scroll_by`) and `:146` (`handle_wheel`).

Scope, measured: the port's subtree only — nodes outside it stay hittable, `handle_pointer` returns the container itself rather than nothing, and an explicit `set_scroll_top(port, 0.0)` fully recovers. No amount of scrolling or re-rendering does. Realistic entry is a host or app computing a delta and passing it to the public `scroll_by`/`handle_scroll`; winit will not emit NaN, and `pan_recognizer.rs:113-118` already guards `dt <= 0.0`, so the internal fling path cannot manufacture one.

**Effort:** S. **Done means:** `shift` opens with the same `is_finite` guard as `set`, and a test passes NaN through `handle_wheel` and asserts the container is unmoved and still hittable.

#### A10. Five windowed demos hand-roll pointer events without `.as_trusted()`, so nothing in them is clickable

```
--- trusted = false (== what the 5 windowed examples build) ---
  button up dispatch = PointerDispatch { target: Some(NodeId(4294967297)), hover_changed: false,
      focus_changed: false, command: None, current_target: None, trust: Untrusted }
  Button: clicks=0 resolved_command=None
  Checkbox: checked=false
--- trusted = true ---
  button up dispatch = PointerDispatch { target: Some(NodeId(4294967297)), ...,
      command: Some(InternedSymbol(114910)), current_target: Some(NodeId(4294967297)), trust: Trusted }
  Button: clicks=1 resolved_command=Some(InternedSymbol(114910))
  Checkbox: checked=true
```

The hit test **succeeds** in both runs — the event lands on the right node and is then discarded. That is the silent-failure shape: hover, `:active`, focus and drag-selection are ungated, so the demos look alive and answer nothing. They also lose long-press/pan/fling (`gesture_arming.rs:90`).

`windowed_counter.rs:125-130`, `controls_window.rs:196-201`, `showcase_window.rs:232-237`, `uikit_window.rs:156-161`, `wireframe_live_window.rs:216-221` all build `PointerEvent::new(...)`, which is `Untrusted` by construction (`crates/domain/obscur-interaction/src/pointer/pointer_event.rs:88`). `grep -rl as_trusted crates/facade/obscur-kit/examples/` returns nothing across all 49 examples. The gate is `crates/application/obscur-runtime/src/runtime/pointer_release.rs:29-30`, `clicked: pressed.filter(|pressed| Some(*pressed) == target && event.trust().is_trusted())`, and it is correct — its own doc lists "an untrusted gesture" as a non-click.

The stated reason for hand-rolling is **factually stale**. Each example's `pointer_button_event.md` says `PointerConverter` "only exposes public builders for Move/Enter/Leave (a button transition *at a position* is the backend's job)". `PointerConverter::pressed` (`pointer_converter.rs:100`) and `::released` (`:114`) are both `pub`, both funnel through `event_of` (`:398`) which calls `.as_trusted()`, and the facade's own `App` runner already uses them (`crates/facade/obscur-kit/src/app/runner.rs:424-437`). `App`-based demos are clickable; these five are not, for no recorded reason.

**Effort:** S. **Done means:** the five examples call `converter.pressed`/`released`, the five stale `.doc` files lose the false rationale, and clicking the counter in `windowed_counter` increments it.

#### A11. Two headless proof examples have been panicking since 2026-07-21

```
$ cargo run -p obscur-kit --example controls_headless
== obscur-kit interactive control gallery — headless proof (CPU) ==
[panel 1] idle captured
thread 'main' (3504152) panicked at crates/facade/obscur-kit/examples/controls_headless.rs:59:5:
checkbox toggled to checked
EXIT=101

$ cargo run -p obscur-kit --example wireframe_live_headless
== obscur-kit interactive Wireframe Kit — headless proof (CPU) ==
[ua] installed default sheet: 53 rule(s) at SystemDefault
[panel 1] idle captured
thread 'main' (3505019) panicked at .../wireframe_live_headless.rs:78:5:
checkbox toggled to checked
EXIT=101
```

Same cause as A10. Proved by one token: a verbatim copy of `controls_headless.rs` with `.as_trusted()` appended at line 259 runs to completion —

```
[panel 1] idle captured
[panel 2] hover + active + focus + open select captured
[panel 3] typed + selected + closed captured
[check] disabled button click is a no-op
```

Neither writes its PNG: after a full run of all 32 default-feature examples, 41 proof PNGs exist under `target/` and `controls_gallery_proof.png` / the wireframe-live proof are both "No such file or directory". `controls_headless.doc/_module.md` opens "**Headless proof: the interactive control gallery WORKS without a display**" and promises "a regression fails the run rather than producing a wrong image."

Blamed to `a34ab2bf` 2026-07-21, "feat(security): enforce the capability gate and the event-trust gate" — red for 38 days. The test suite was updated in that same commit and says so in prose (`tests/widget_click.rs:43-46`); the examples were missed. CI builds exactly one example (`.github/workflows/ci.yml:71`) and no `xtask` step runs any.

Blast radius is exactly these two: only the click default action is trust-gated, so the other five pointer-driven proofs (`scroll_proof`, `textarea_proof`, `caret_scroll_proof`, `textarea_vscroll_proof`, `inspect_overlay_proof`) still exit 0 and write their PNGs.

**Effort:** S. **Done means:** `.as_trusted()` at `controls_headless.rs:259` and `wireframe_live_headless.rs:421`, both examples exit 0 and write their PNGs, and CI runs every example that can run headless.

---

### Part B — wrong pixels, wrong numbers

#### B1. The two backends interpolate gradient stops in different colour spaces

```
$ /tmp/probe-renderers/target/debug/grad_probe
linear-gradient(red, green) across 64px, sampled along the middle row:
  x= 0  cpu=(253,  1,  0)  vello=(255,  0,  0)
  x=16  cpu=(189, 33,  0)  vello=(212, 80,  0)
  x=32  cpu=(126, 65,  0)  vello=(166,105,  0)
  x=48  cpu=( 62, 97,  0)  vello=(113,120,  0)
  x=63  cpu=(  2,127,  0)  vello=( 13,128,  0)
```

Diagnosed, not inferred: computing both candidate ramps from scratch against css-color-4 §9.2 matrices at pixel centres gives `srgb_lerp` = [253,1,0] / [189,33,0] / [126,65,0] / [62,97,0] / [2,127,0] and `oklab_lerp` = [254,9,0] / [211,81,0] / [165,106,0] / [111,120,0] / [14,128,0]. The CPU column matches the sRGB lerp to the unit at all nine samples; Vello matches Oklab within 1–3 units.

Same cause on the other brushes:

```
--- linear gradient black->white: max channel delta 35, 1024/4096 pixels differ by >2
      (16,20): cpu=(68,68,68) gpu=(34,34,34)        <- a factor of 2 in luminance
--- conic gradient: max delta 105, 2284/4096 off by >3
      (32,50): cpu=(130,125,0) gpu=(208,169,0)
--- repeating linear gradient: 4096/4096 off by >3 (ramp positions agree exactly, only colours differ)
--- dashed stroke: max delta 0, 0/4096   <- control, backends agree
```

The CPU midpoint (126,65,0) is, to within one unit, the number `crates/adapters/obscur-render-vello/src/conversion/brush_converter.doc/GRADIENT_INTERPOLATION_CS.md` names as the bug it was fixed to avoid: "peniko itself defaults a `Gradient` to sRGB … which produces the muddy dark midpoint of e.g. `linear-gradient(red, green)`" … "the tag alone left the red→green midpoint at the muddy sRGB `(127, 64, 0)`".

`crates/adapters/obscur-render-cpu/src/renderer/brush_fill.rs:115` (`lerp_color`) and `:122` (`premul_lerp_color`) mix raw sRGB channels, used by `fill_linear` (`:140`) and `fill_conic` (`:19`). Versus `brush_converter.rs:12` (`GRADIENT_INTERPOLATION_CS = ColorSpaceTag::Oklab`) and `:105` (`stops`, densifying through `premul_oklab`/`oklab_lerp`).

Not covered by the CPU/GPU policy: `cpu_renderer.doc/_module.md:29-32` enumerates the gradient approximation as exactly one thing — "the **radial** brush still degrades to a flat solid of its first stop" — and says linear and conic "rasterize for real"; its determinism boundary says "only rasterization, anti-aliasing, and precision differ", which a hue shift is not. `grep -rn -i oklab crates/adapters/obscur-render-cpu/` → zero hits, though `obscur-scene`'s `convert.doc/linear_srgb_to_oklab.md` says the forward conversion exists precisely because "densifying a gradient's stops in Oklab needs this forward direction". `docs/servo-algorithm-catalog.md:181` row F-4 already records "gradient/mix interpolation space … default **Oklab**" as `[MISSING — gradient gap]`.

**Effort:** M. **Done means:** the CPU mixers densify in Oklab through the primitive `obscur-scene` already ships, `linear-gradient(red, green)` measures the same midpoint on both backends, and a test pins one sample.

#### B2. `VelloRenderer::render_to_image` returns straight alpha while its doc promises premultiplied

```
$ /tmp/probe-renderers/target/debug/png_probe
scene: one rect, straight colour rgba(0.2,0.4,0.6,0.5)
       => an honest straight-alpha PNG pixel is (51, 102, 153, 128)

cpu  raw pixmap byte  (16,16) = [26, 51, 77, 128]
cpu  PNG pixel        (16,16) = [52, 102, 153, 128]

vello raw readback byte (16,16) = [52, 102, 153, 128]
vello PNG pixel         (16,16) = [104, 203, 255, 128]
```

Not ambiguous: premultiplied-in-encoded-space would be 26, premultiplied-in-linear-space 35; 52 is exactly the straight sRGB byte. 52·255/128 = 103.6, 102·255/128 = 203.2, 153·255/128 = 304.8 → clipped 255 — tiny_skia's demultiply-on-encode applied to data that was never multiplied.

`crates/adapters/obscur-render-vello/src/renderer/vello_renderer.doc/render_to_image.md:17-18` says "The returned bytes are **premultiplied** RGBA8 (Vello's texture output), so they can be handed straight to e.g. `tiny_skia::Pixmap::from_vec`." Four other files in the same crate say the opposite, unprompted: `premultiply_blit.wgsl:3` ("Vello writes STRAIGHT alpha"), `premultiply_blit.doc/PremultiplyBlitter.md:3-4` (which records this exact defect being found and fixed once before on the Wayland present path — "a card authored at `alpha: 0.72` over a dark desktop measured `(255, 252, 249)` where an honest blend is `(187, 183, 181)`"), `effects/color_matrix.doc/_module.md:20`, `effects/gaussian_blur.rs:360`. The on-screen path has a whole WGSL pass for this; the read-back path at `vello_renderer.rs:455-462` copies the texture bytes verbatim.

**Blast radius is one row of one example, not eleven.** All 11 `*_vello_png` examples were built and run and their PNGs measured by alpha histogram: ten contain **zero** partial-alpha pixels (each paints an opaque full-viewport background), so demultiply divides by 1 and they are bit-identical to a correct render. Only `wireframe_kit_vello.png` is affected, on row 6656: 2019 pixels, two values `(255,255,249,227)` and `(255,255,255,196)` — the antialiased bottom page edge pushed to white and clipped. 0.014% of the image.

The suite cannot see it: `tests/fractional_edge.rs:246 the_gpu_backend_keeps_the_same_extreme_pixels` is `#[ignore = "needs a GPU adapter"]`, returns early without an adapter, and asserts only coverage — no channel value is ever compared across backends.

**Effort:** S. **Done means:** either the un-padding copy multiplies rgb by alpha, or `render_to_image.md` and the 11 call sites are corrected — and a test compares one translucent pixel across the two backends.

#### B3. `GlyphRun::advance()` drops a control character's glyph; `white_space_proof` panics on `tab-size: 8` and passes `tab-size: 4` by 0.431 px of coincidence

```
$ cargo run -p obscur-kit --example white_space_proof
[block] 1 line(s): [22.5]
...
thread 'main' (3504871) panicked at crates/facade/obscur-kit/examples/white_space_proof.rs:163:5:
D(tab-size:8): pen after tab (22.5) must land on 8·space (45.8)
EXIT=101
```

Measured from the display list (DejaVu Sans 18px, space advance 5.7217, content `"a\tb"`, `white-space: pre`), `run.advance()` beside the run's real painted extent `max(g.x() + g.advance())`:

```
space-only   glyphs=1 run.advance()=  5.722 painted_extent=  5.722
tab-size:1   glyphs=2 run.advance()= 22.456 painted_extent= 22.869
      cluster=0 x= 0.000 adv=11.030   cluster=2 x= 11.443 adv=11.426
tab-size:4   glyphs=2 run.advance()= 22.456 painted_extent= 34.312   cluster=2 x= 22.887
tab-size:8   glyphs=2 run.advance()= 22.456 painted_extent= 57.199   cluster=2 x= 45.773
tab-size:16  glyphs=2 run.advance()= 22.456 painted_extent=102.973   cluster=2 x= 91.547
plain-ab     glyphs=2 run.advance()= 22.456 painted_extent= 22.456
a-space-b    glyphs=3 run.advance()= 28.178 painted_extent= 28.178
```

Cluster 1 is absent at every tab-size; the run for `"a\tb"` is byte-for-byte the run for `"ab"`. A 4.6× under-report at tab-size 16. The tab-stop math itself is **correct** — 'b' lands on 2·/4·/8·/16·space exactly — so this is a wrong reported number, not a wrong picture.

The false pass is real and the example is worse than it looks: `advances[0]+advances[1]` = 11.030+11.426 = 22.456 against `4*space_adv` = 22.8868 with a 0.5 px tolerance — **0.4308 px** of margin, on a sum containing no tab at all. The `advances.len() >= 2` gate at `:150` lets a 2-glyph run through where 3 were intended. Two further assertions at `:164` would also fail if execution reached them: `22.456 > 22.456` and `22.456 > 28.178` — the tab lines report *narrower* than the single-space line.

`crates/application/obscur-runtime/src/orchestration/text_painter.rs:49-68` — `let any_control = content.chars().any(char::is_control);` … `if !suppressed { glyphs.push(...) }` … `pen_x += glyph.x_advance();`. The advance survives in the pen; the glyph is dropped from the run. Consumed by `crates/domain/obscur-scene/src/text/glyph_run.rs:101-103`. The suppression is deliberate and documented — but `text_painter.doc/positioned_run.md` promises the opposite of what happens: "the suppression is purely visual, leaving byte offsets, **advances**, and line breaks untouched."

Scope, measured: no runtime or renderer code reads the scene `GlyphRun::advance()` (`text_painter.rs:285,390` call obscur-text's `ShapedRun::advance`, which *does* include the tab), so layout, line boxes, caret walk and rasterization are unaffected. What is wrong is the number a facade consumer reads back, and the example that reads it. Newline-only content does not diverge (a line-terminating newline is the last glyph, so the sum agrees); an interior tab is the case that bites.

**Effort:** S. **Done means:** either the suppressed glyph is emitted with zero ink or `GlyphRun` carries a run advance independent of its glyph list; `white_space_proof` exits 0; and its `tab-size: 4` assertion measures something a tab-free run could not satisfy.

#### B4. `narrow_f32` saturates to ±inf with no diagnostic, and the transform fold then manufactures NaN in coefficients the author never wrote

```
$ /tmp/probe-parsers/target/debug/nums2
transform: translate(1e39px)  -> errors=0 unsupported=0
   Transform { matrix: [1.0, 0.0, 0.0, 1.0, inf, NaN], is_none: false }
transform: rotate(1e39deg)    -> Transform { matrix: [NaN, NaN, NaN, NaN, 0.0, 0.0] }
transform: scale(1e39)        -> Transform { matrix: [inf, NaN, NaN, inf, 0.0, 0.0] }
transform: scaleX(1e39)       -> [inf, NaN, 0.0, 1.0, 0.0, 0.0]
transform: matrix(1e39,0,0,1,0,0) -> [inf, NaN, 0.0, 1.0, 0.0, 0.0]
width: 1e39px                 -> Length(Px(LogicalPx(inf)))
font-size: 1e39px             -> Length(Px(LogicalPx(inf)))
filter: blur(1e39px)          -> Filter([Blur(inf)])
flex-grow: 1e39               -> Number(inf)

$ /tmp/probe-parsers/target/debug/consequence
translate(1e39px) matrix maps rect ->
  Rect { origin: Point { x: inf, y: inf }, size: Size { width: NaN, height: -inf } }
  point (10,10) -> Point { x: inf, y: NaN }
```

`1e400` and a 41-digit integer give the same; so does the finite-in-f32 `transform: scale(3.5e38)`.

Two separate wrongs. (a) `crates/frontend/obscur-css/src/compiler/property_compiler.rs:258-260` — `narrow_f32` is a bare `value as f32`, while `property_compiler.doc/narrow_f32.md` calls it "this single checked-narrowing point [that] keeps the (intentional) truncation contained". It is not checked and does not truncate. `z-index: 1e400` is already refused as unsupported; lengths are not. (b) Even granting inf, `translate(1e39px)` names only an x offset, so `f` must be 0.0 — the NaN is manufactured by composing `AffineTransform::IDENTITY` with the function, where `0.0 * inf` is NaN. `affine_transform.rs:63-76` (`multiply`), driven from `transform_lowering.rs:33-41` (`flatten_or_defer`). `transform_lowering.doc/_module.md` describes the fold and says nothing about non-finite input. The element then has NaN paint and hit geometry rather than merely being scaled off-screen.

Related, and the same class: `Dimension::resolve` (`crates/domain/obscur-layout/src/model/dimension.rs:27`) computes `base * percent / 100.0`, so a 100% child of a large finite parent overflows —

```
[ok ] parent width:3.4e36px, child 100%: (Some(3.4e36), Some(3.4e36))
[ok ] parent width:1e37px,   child 100%: (Some(1e37),   Some(inf))
```

The obvious reassociation is **not** the fix: `base / 100.0 * percent` still yields `inf` for 100% of `f32::MAX`, turns 100% of 1e30 into 1.0000001e30, and flushes a denormal base to 0. What is missing is a finite clamp on the resolved length, which the kit has nowhere.

**Effort:** S. **Done means:** `narrow_f32` clamps to `f32::MAX`/`MIN` or refuses the declaration; the transform fold cannot produce a NaN from finite-signed author input; resolved lengths are clamped finite; and `narrow_f32.md` stops claiming a check that is not there.

*(Not adversarially re-run — measured once.)*

#### B5. An SVG arc with a large radius produces NaN path points, defeating the parser's own finiteness filter

```
$ /tmp/probe-parsers/target/debug/consequence
M0 0 A1e18 1e18 0 0 1 100 0  -> Some(CubicTo { control1: (-9.999999e17, -5.5228486e17), … })
M0 0 A2e19 2e19 0 0 1 100 0  -> Some(CubicTo { control1: Point { x: NaN, y: NaN },
                                               control2: Point { x: NaN, y: NaN },
                                               end:      Point { x: NaN, y: NaN } })
M0 0 A1e20 1e20 0 0 1 100 0  -> all NaN
M0 0 A1e30 1e30 0 0 1 100 0  -> all NaN
$ (earlier run) M0 0 A1e30 1e30 0 0 1 100 0 -> Ok(2 segs) nan_or_inf=true
   bounds=Some(Rect { origin: (0,0), size: Size { width: 0.0, height: 0.0 } })
```

`parse_path_data` returns `Ok` in every case, and the NaN silently collapses `bounds()` to a 0×0 rect. The correct output is close to the chord (0,0)→(100,0) — a shallow sliver of a huge circle.

The parser deliberately refuses non-finite *input* numbers (`crates/domain/obscur-scene/src/path/path_data.rs:371`, `.filter(|value| value.is_finite())`), so accepting the input and emitting NaN geometry one function later breaks its own invariant. Cause is f32 overflow in the endpoint→centre conversion at `:236-260`: `rx*rx*ry*ry` overflows above rx ≈ 1.8e19, `numerator` becomes `inf - inf` = NaN, and `.max(0.0)` launders it to 0.0 while `cx`/`cy` still come out NaN. The only in-tree consumer, `crates/components/obscur-widgets/src/progress/progress_painter.rs:92`, does `.ok().filter(|path| !path.is_empty())` — so it accepts the NaN path and hands it to the renderer.

**Effort:** S. **Done means:** the F.6 conversion runs in f64 (or the radii are clamped), and a large-radius arc produces finite points and a real `bounds()`.

*(Not adversarially re-run.)*

#### B6. Compressed SVG arc flags — the form SVGO emits — are rejected, so an optimized path silently draws nothing

```
$ /tmp/probe-parsers/target/debug/arcflags
M0 0 A50 50 0 1 1 100 0  -> Ok [MoveTo(0,0), CubicTo{…}, CubicTo{…}]
M0 0 A50 50 0 11 100 0   -> Err PathDataError { at: 22, what: MissingNumber }
M0 0 a50 50 0 0150 0     -> Err PathDataError { at: 20, what: MissingNumber }
M0 0 A50 50 0 0150 0     -> Err PathDataError { at: 20, what: MissingNumber }
M0 0 A50 50 0 1 1100 0   -> Err PathDataError { at: 22, what: MissingNumber }
```

All four rejected forms are well-formed SVG: the path grammar defines `flag: "0" | "1"` as a single character and makes the separator after each flag optional. `A50 50 0 11 100 0` is rx=50 ry=50 rot=0 large-arc=1 sweep=1 end=(100,0). SVGO's `convertPathData` is on by default and emits exactly this, so any icon or progress track run through a standard optimizer fails to parse.

`crates/domain/obscur-scene/src/path/path_data.rs:160-168` — the `b'A'` arm reads all seven operands with `self.number()`, which at `:337` scans a maximal digit run and swallows `11` as one number. The two flags need a single-character reader. Consequence is silent: `progress_painter.rs:92` turns the `Err` into "no track drawn", with no diagnostic anywhere.

**Effort:** S. **Done means:** a dedicated flag reader, the four inputs above parse identically to their spaced forms, and `progress_painter` reports a path it could not parse.

*(Not adversarially re-run.)*

#### B7. Past 256 tree levels, nodes silently get no box and no paint — and an auto-height ancestor at depth 1 collapses to 0

```
$ /tmp/probe-degen/target/debug/pc
depth 253: nodes=255 with-box=255 leaf=Some((10.0, 10.0)) diagnostics=0
depth 254: nodes=256 with-box=256 leaf=Some((10.0, 10.0)) diagnostics=0
depth 255: nodes=257 with-box=256 leaf=None               diagnostics=0
depth 260: nodes=262 with-box=256 leaf=None               diagnostics=0

$ (independent probe: red leaf, scene statistics, hit test)
n=254 boxed=256 leaf_box=Some((10.0,10.0)) leaf_is_target=true
      target=Some("depth=255 classes=[\"leaf\"]")
      scene=SceneStatistics { command_count: 1, draw_count: 1, ... } diags=0
n=255 boxed=256 leaf_box=None              leaf_is_target=false
      target=Some("depth=255 classes=[\"d\"]")
      scene=SceneStatistics { command_count: 0, draw_count: 0, ... } diags=0

$ (auto-height ancestor: .root{width:100px}, leaf 10x10)
n=253 root(w,h)=Some((100.0, 10.0))
n=254 root(w,h)=Some((100.0,  0.0))     <- a node at DEPTH 1 reports the wrong height
n=300 root(w,h)=Some((100.0,  0.0))
```

`draw_count` 1 → 0: the fill is genuinely not emitted. The press lands on the ancestor. `diags=0` with a collecting `DiagnosticSink` installed. And the measure side returns `Size::ZERO`, so the zero propagates all the way up — this is not only an invisible tail, it is a wrong number at a shallow, on-screen container.

Both bails are bare silent returns: `crates/application/obscur-runtime/src/orchestration/arrange_pass.rs:70` (`if depth == 0 { return; }`) and `measure_pass.rs:95` (`return Size::ZERO;`), budget from `layout_coordinator.rs:28` (`MAX_DEPTH: u32 = 256`) passed in at `:303`. `layout_coordinator.doc/MAX_DEPTH.md` says verbatim: "It is deliberately generous (far deeper than any realistic UI tree) so it never truncates a well-formed tree." Every run above is a well-formed acyclic tree that it truncates.

256 is not stack-tuned — depth 254 completes on an 8 MB main-thread stack with no sign of pressure, so raising it is available. The load-bearing fix is the missing `report(...)` this codebase emits for every other silent degradation (`unsupported_effects`, `css.unsupported_at_rule`, `css.invalid_value`); today an author gets a green compile, a blank region, a dead click target and a container measuring 0.

**Effort:** S. **Done means:** exhausting the budget reports a diagnostic, `MAX_DEPTH.md` stops claiming non-truncation, and a test asserts the diagnostic fires at 257.

---

### Part C — crashes and hangs on hostile input

#### C1. `Reconciler::reconcile_into` recurses without a depth cap and aborts the process

```
$ ulimit -s
8192
$ /tmp/probe-degen/target/release/p6 10000 full
built 10000

thread 'main' (3554024) has overflowed its stack
fatal runtime error: stack overflow, aborting
exit=134
```

SIGSEGV, not a catchable panic — `catch_unwind` does not see it. gdb on the release binary:

```
#3  obscur_tree::reconciliation::reconciler::apply_props ()
#4  obscur_tree::reconciliation::reconciler::Reconciler::reconcile_into ()
#5..#11 ...::reconcile_into ()   (identical frame, repeating)
```

Thresholds, binary-searched: release / 8 MiB main stack — n=9000 lays out 9002 nodes cleanly, n=10000 aborts. Release under `ulimit -s 2048` (a default `std::thread` stack) — n=2000 fine, n=2500 aborts. ~880 bytes per level. Building and dropping the same 50 000-deep `View` without rendering is fine, so the recursion is the toolkit's, not the fixture's. At n=9000 the whole `render_now` + cascade + layout + `inspect_snapshot` pipeline completes, so reconciliation is the *binding* constraint: capping this one walk fixes the shape.

`crates/domain/obscur-tree/src/reconciliation/reconciler.rs:205` — `self.reconcile_into(tree, id, children.nodes(), plan);` — entered from `:48` (`reconcile_root`) and `:61` (`reconcile`). `grep -rn MAX_DEPTH crates/` finds a cap only on the layout side; `reconcile_into.md` reads in full "Recurses into each element child. A no-op if `parent` is stale."

The layout precedent's rationale only half-transfers: `MAX_DEPTH.md` argues *cycle* safety over the `RetainedTree`, and the reconciler walks the owned `ViewNode` tree, which cannot be cyclic. Only the "never overflows the stack" half applies. Reachable from ordinary public API (`ViewNode::element(...).with_children(...)` + `render_now`) — not from a hand-authored `.okui`, but from a recursive component over unbounded data.

**Effort:** S. **Done means:** a depth counter that stops recursing and reports, and a 20 000-deep view returns a diagnostic instead of SIGSEGV.

#### C2. The CSS value parser has no recursion cap: ~6 KB of `(` aborts the process

```
$ ./target/release/vp 20000
[ValueParser::parse on 20000 bytes of '(' ]
ERR CssParseError { kind: UnexpectedEof, message: "unexpected end of input", span: SourceSpan { start: 20000, end: 20000 } }
exit=0

$ ./target/release/vp 40000
thread 'main' (3578309) has overflowed its stack
fatal runtime error: stack overflow, aborting
exit=134

$ ./target/release/thr 5000      # 2 MiB spawned thread, div{width:(((…1px}
OK rules=1 errors=2   exit=0
$ ./target/release/thr 6000
thread '<unknown>' (3594073) has overflowed its stack
fatal runtime error: stack overflow, aborting

$ ./target/release/inline 40000  # compile_inline_style("width:((((…))))")
thread 'main' (3578351) has overflowed its stack
fatal runtime error: stack overflow, aborting
exit=134
```

Purely a depth cliff — correct behaviour one step below it. Debug value parser 4 000–8 000; debug inline 5 000–6 000; release 2 MiB thread 5 000–6 000; release main thread 20 000–40 000, i.e. a ~30 KB theme file.

`crates/frontend/obscur-css/src/parser/value_parser.rs:78` (`parse_components`) mutually recurses with `:106` (`parse_component`); the bare-LParen arm at `:150-157` and the function-call arm at `:116-119` both recurse with no depth argument. `grep depth` over `crates/frontend/obscur-css/src/parser/` finds only flat i32 counters in iterative recovery scans (`stylesheet_parser.rs:482-575`, `media_query_parser.rs:279`, `selector_parser.rs:340`) — none guards the recursive descent.

**Second site, same class:** nested `@media`/`@container` blocks. `crates/frontend/obscur-css/src/parser/stylesheet_parser.rs:156` — `parse_rules(&mut self, nested: bool)` recurses at `:293` and `:315`; `nested` is a bool, not a counter.

```
$ ./target/release/thr 2000 media
OK rules=1 errors=2000          <- correct one step below the cliff
$ ./target/release/thr 4000 media
fatal runtime error: stack overflow, aborting   exit=134
$ ./target/release/deep css-media 20000         # balanced braces, main thread
fatal runtime error: stack overflow, aborting
```

The realistic vector is not the `style` attribute (author-controlled, and no non-main thread in `crates/` parses CSS) but the public facade API `AppControl::set_stylesheet(css: impl Into<String>)` (`crates/facade/obscur-kit/src/control/app_control.rs:75` → `css_pipeline.rs:74`), which accepts arbitrary CSS at runtime — a user theme file or a DevTools live edit.

Every other recovery path in this parser is explicitly hardened against non-progress loops, with the reasoning written down (`stylesheet_parser.rs:161-166`: "which is a stylesheet taking the process down, not a diagnostic"; `path_data.rs:98-105`: "it takes the frame thread and then the machine's memory with it"). Nothing guards depth. `docs/unfinished.md:327-333` predicts this class ("no fuzzing, no Miri, no sanitizers", naming two previous hand-found parser DoS commits) but names the missing CI job, not these two defects.

**Effort:** S. **Done means:** a depth counter in `parse_components` and a nesting counter in `parse_rules`, each returning a `CssParseError` past a hard limit; 100 000 `(` and 100 000 nested `@media` both produce a diagnostic.

#### C3. `set_viewport_scroll` stores a NaN its sibling refuses

```
$ /tmp/probe-degen/target/debug/p1
[ok ] set_viewport_scroll(NaN,0): ((NaN, 0.0), Some((0.0, 0.0)))
[ok ] set_content_offset(NaN,0):  ((0.0, 0.0), Some((0.0, 0.0)))
[ok ] set_viewport_scroll(inf,0): ((inf, 0.0), Some((0.0, 0.0)))
[ok ] set_content_offset(inf,0):  ((0.0, 0.0), Some((0.0, 0.0)))
```

`surface_geometry.rs:88-90` is a `const fn` storing whatever it is handed; `:74-80` drops non-finite components per axis and says so in its public doc ("Non-finite components are ignored"). `set_viewport_scroll.md` is silent.

Measured effect on layout: **none.** A NaN scroll yields the identical arrangement to scroll 0 (sticky child at y=50, siblings unmoved), inf and 1e30 clamp correctly to 360, and a following sane `set_viewport_scroll(0,100)` recovers fully. The only residue is `document_viewport()` → `resource_resolution.rs:52` → `resource_bridge.rs:714/724`, where a NaN rect makes `rects_intersect` false and every image is marked `VisibilityPriority::Offscreen`. That priority is read only by `image_cache.rs:110` and `cache/eviction_policy.rs`; there is no visibility-gated lazy load in the tree, so the consequence is degraded eviction ranking while a host is holding a NaN — not deferred image loading.

**Effort:** S. **Done means:** `set_viewport_scroll` filters non-finite components the way `set_content_offset` does, and its doc says so.

---

### Part D — cosmetic

#### D1. Synthetic bold is 1.34× heavier on Vello than on the CPU backend

At 24 px the ink proxy is a weak signal; measured geometrically at 300 px with one glyph on a 512² canvas, bbox at the alpha≥128 half-coverage crossing:

```
font_size = 300
  plain  cpu bbox@128=(129, 166, 292, 379) area=14154.1
         gpu bbox@128=(129, 166, 292, 379) area=14167.4
  bold   cpu bbox@128=(124, 161, 297, 384) area=19358.4
         gpu bbox@128=(123, 160, 298, 385) area=21133.6
  cpu grow per side: left=5 right=5 top=5 bottom=5
  gpu grow per side: left=6 right=6 top=6 bottom=6
  gpu/cpu area gain ratio = 1.339
  predicted from constants: cpu 0.03/2=4.5 px/side, gpu 0.02=6 px/side, ratio 1.333
```

Plain agrees to 0.09% in area and to the pixel on all four sides, so nothing here is AA noise. Size-independent, not the 10.8% the 24 px proxy suggests.

Cause confirmed in dependency source, not inferred: tiny_skia's `Stroke { width: font_size * 0.03 }` is **centred** on the filled outline, so 0.015 em/side (`crates/adapters/obscur-render-cpu/src/renderer/glyph_raster.rs:69`, applied `:86-89`), whereas Vello's `FontEmbolden` amount goes to `kurbo::expand_path` (`vello_encoding-0.9.0/src/glyph_cache.rs:206-216`), documented at `kurbo-0.13.1/src/expand.rs:110-126` as an outward offset **per side**, so 0.02 em/side (`crates/adapters/obscur-render-vello/src/conversion/scene_converter.rs:28`, applied `:320-321`).

Neither constant's doc mentions the other backend. The italic pair does: `FAUX_ITALIC_SKEW.md` ends "Matches the CPU backend's factor so both fake the same slant." That sentence exists for the constant that agrees and is missing from the one that does not. No test pins either.

**Effort:** S. **Done means:** the CPU fraction becomes 0.04 or the Vello fraction 0.015, a test pins the pair, and the surviving doc says it matches the other backend.

---

### What was cleared

Nothing else in the repository records which areas survived execution. This is the other half of the pass.

**Examples.** 29 of 32 default-feature examples exit 0 and their self-checks hold: `dashed_border` (10 dash runs / 10 gaps on the top scanline), `border_triangle` (4 centroids inked, 4 corners clear), `outline` (ring outside the box, dashed, offset gap white), `inset_shadow` (edge inked, centre clear, no bleed), `overflow` (exactly 1 PushClip; hidden spill white, visible spill green), `margin_collapse` (all four columns), `stacking_order` (§9.9 both directions), `transform_interp` (45° at full scale, not a naive lerp), `multiline_text`, `text_spacing_transform`, `text_ellipsis` (measure and compose share the same shaped ellipsis glyph), `caret_scroll`, `textarea`, `textarea_vscroll`, `scroll_proof`, `css_motion`, `radii_geometry`, `counter_headless`, `showcase_png`/`okui`/`okui_codegen` (the `.okui` bridge and build-time codegen produce byte-identical 188 559-byte PNGs), `uikit_png`, `wireframe_png`, `capabilities_png`, `inspect_overlay`, `showcase_inspect`, `ua_default_proof` (bottom panel contains 11 211 `#c00` pixels, top panel none), `ua_widgets_proof`, `pseudo_state_proof`. All 11 GPU proofs run clean on real hardware. All 10 windowed/App examples boot without panicking; `showcase_window` and `wireframe_live_window` present at 64–107 fps and open their DevTools endpoints. Two PNGs that looked degenerate (2 distinct colours) are genuine hard-stop stripe patterns; one that looked half-black is `(0,0,0,0)` unpainted page area.

**Backend agreement — byte-identical, max channel delta 0 over the whole buffer:** opaque solid fills; src-over of a translucent rect over an opaque one; strokes (width 4, and under `scale(2,2)` — the CTM scales the width identically); dashed strokes (`with_dash([8,8], 0)`, phase and gaps on the same pixels); NonZero **and** EvenOdd fill rules on a self-intersecting path; non-uniform `scale(2, 0.5)`; nested clip intersection; a clip pushed inside a transform; **all eleven blend modes** over an opaque backdrop (Multiply/Screen/Overlay/Darken/Lighten/Difference/Exclusion/ColorDodge/ColorBurn/HardLight/SoftLight); a `DrawImage` whose source is `AlphaMode::Premultiplied` (both yield straight (52,102,153,128)).

**Extreme pixels (audit 2-J).** A rect exactly covering the buffer is solid to the corners on both. `rect(0.5,0.5,63,63)` gives alpha 128 on the outer row/column on both (max delta 1). A 0.25 px rect gives alpha 64/65 on both. On a fractionally-scaled tangent circle (89.6 px shape in a 90 px buffer) both keep the extreme columns — mid-row span (0, 89) on both. Residual is AA coverage only.

**Text placement.** With real NotoSans outlines the ink bounding box is identical on both backends for a plain run (14,42)-(87,64), for synth-oblique, and under `scale(2,2)`.

**Spec conformance, by execution.** Grid `minmax(200px,100px)` floors the max by the min. A single alignment subject under `space-around`/`space-evenly` lands at x=150 in a 400 container, `space-between` at x=0. `align-items: center`/`flex-end` on an item taller than the line give y=-50/-100 (this is the control that convicts the main axis). `align-items` initial is `stretch`. Grid `auto auto` tracks stretch to 300/300. `margin-left: 25%` → 100 against a 400 CB. `width: 50%` with `min-width: 75%` clamps to 300. `box-sizing: border-box` with 20+20 padding keeps a 100 border box. `calc(100px + 20px)` resolves. A relatively positioned box with `left:-20px; right:50px` honours `left` and ignores `right`. By reading: the selectors-4 §15 specificity model (`:is()`/`:not()`/`:has()` take the whole most-specific complex selector; `:where()` is zero; `:nth-child(An+B of S)`; `::part()` as a pseudo-element) matches the implementation, and the `flex` shorthand expansion matches css-flexbox-1 §7.1.1 including the `flex: <number>` → `<n> 1 0px` default that implementations usually get wrong.

**Hostile input, cleared as correct.** Transformed hit testing (`translate(1000px,0)`, `scale(0.001)` about the default 50% origin, `scale(0)` — all resolve the inverse and hit/miss correctly). NaN/inf/-1e30 pointer positions (rejected; the next good move recovers hover). `set_scroll_top` (guards `is_finite`, clamps negatives to 0 and 1e30 to max). `set_content_offset`. The 0.1 `MIN_DEVICE_SCALE` clamp. `SliderScale` (`is_degenerate`, `stepped_by`, `quantize`, `value_at` all guard min≥max, step≤0, non-finite). `grid-template-columns: repeat(1000000, 1fr)` (capped, 4 ms). Duplicate `id` attributes. Class names containing `\n`, `}`, empty, whitespace-only, 100 k characters. Text: empty, whitespace-only, a lone U+0301, five stacked combining marks, U+FEFF, an embedded NUL, RTL/LTR mixed, a U+202E override, a ZWJ family emoji, 500 soft hyphens, a 10 000-character unbroken word (one line, 147 ms), a 100 k-character paragraph (4167 lines, 11.9 s debug — slow, not a hang). 10 000 siblings (1.0 s debug). Inline `style` containing `};`, and a 260 000-character one. Percentages against a zero basis (0, not NaN). NaN and inf viewport dimensions (0 out, no NaN propagation).

**Parsers: no panic and no hang in 6.1 million generated inputs** (523 328 token-soup cases plus 5.6 M mutation cases over eight CSS sheets, three inline styles, three SVG paths and four `.okui` documents, with a 40-token poison set). Worst single case: 904 µs release, 2.3 ms debug. Specifically cleared: the two historical infinite-loop sites are fixed and their nets hold (`M0 0 {cmd} !` for all 20 command letters, `Z 1`, `z 5`, a stray top-level `}`, `}`×200, `{`×200, `@x`×200, `(`×200 all terminate with a diagnostic); the CSS tokenizer on unterminated strings, unterminated comments, a trailing backslash inside a string, and `\\` runs (the `pos += 2` escape skip can overshoot but `slice`/`byte` clamp); `find_lbrace`/`recover` letting brace depth go negative on a stray `)` self-heals (`) {} p{color:red}` still yields the `p` rule); `SelectorParser` is iterative — `div` + `:not(`×50000 parses in one pass with no stack growth; the `.okui` front end handles every malformed shape with a *named* diagnostic (`ExpectedClose`, `MismatchedCloseTag` naming both tags, `ExpectedAttributeValue`, `UnterminatedExpression`, stray `}`, `</div>` with no open, `<>`, non-ASCII tag name); okui nesting is quadratic but never crashes (800 nested divs 2.4 s debug; 5 000 nested tags parse and cast); `z-index: 1e400`, `calc(1px / 0)` and `calc(0 / 0)` are correctly refused; `width: NaN` and `width: infinity` are treated as ordinary unknown keywords.

---

### Corrections owed to Pass 1

`docs/unfinished.md` item 27 says: "Checked today: `cargo check -p obscur-kit --features full --examples` exits 0, so nothing has rotted yet — this is risk, not decay." That line is false and should be struck. Two examples have failed at *runtime* since 2026-07-21 while the compile check stays green, which is exactly why the CI gap in item 27 costs more than that sentence concedes.

`docs/unfinished.md:240` says of the deferred keyword initials that "every consumer's fallback arm is the CSS initial — verified for display, box-sizing, direction, align-items, flex-wrap, text-align, text-transform, white-space, overflow-wrap and word-break." `align-content` is absent from that list and is its counterexample (item A6).

---

### Dropped

**Nine candidates were dropped** because an adversary found them deliberate, documented, or measurably inert — plus one false consequence and one false fix that were attached to surviving items:

1. **CPU nearest-neighbour image sampling.** Reproduces exactly (a 4× downscaled checkerboard comes out uniformly 255 against Vello's 128; an independent probe confirms `FilterQuality::Bilinear` at `image_tiling.rs:110` reproduces Vello's 128 to the unit), but it is recorded as deliberate one directory from the blamed line — `image_tiling.doc/draw_image.md:16-17`, "The default nearest sampling keeps the blit crisp and deterministic." The finding's three supports were also false: `image-rendering: auto` is UA-dependent per `specs/w3c/css-images-3.txt:1480-1481` (`smooth` is the separate keyword), and `specs/obscur-kit-2-spec-drafts/05-rendering-text-and-resources.md:43-58` puts rasterization under "Allowed differences". Worth noting anyway for whoever revisits it: `cpu_renderer.doc/_module.md:60-63` says the opposite ("both backends show the same picture"), and on a realistic (non-Nyquist) 4× downscale the measured cost is mean error 13.5/255 nearest against 9.4 bilinear.
2. CPU radial gradients degrading to a flat first-stop colour — stated policy, `brush_fill.doc/_module.md`, spec §5.2.
3. CPU blur / backdrop — CPU/GPU effect-split policy; not probed.
4. The 0.1 `MIN_DEVICE_SCALE` clamp — deliberate, pinned at `surface_geometry.rs:176-198`.
5. `repeat()` track capping — deliberate.
6. `SliderScale`'s degeneracy guards — deliberate.
7. `ensure_started` in the path parser (a path opening with a drawing verb opens at the origin) — documented policy, `path_data.rs:206-210`.
8. Arc large-arc/sweep flags accepting any non-zero number — lenient, not wrong.
9. `flex-basis: -10px`, `outline-width: -3px`, `min-width: -50px`, non-flex `gap: -10px`, `line-height: -2` — measured inert or correct; cut from item A3's evidence so a fix does not chase them.

Also dropped: the claim that a NaN viewport scroll would stop images being lazily loaded (there is no visibility-gated lazy load in the tree), and the proposed `base / 100.0 * percent` fix for the percentage overflow (still overflows at `f32::MAX`, loses precision at 1e30, flushes denormals to 0).

---

### The most alarming thing

Not the biggest. The most alarming is **A2** — an unrecognised value overwriting the valid declaration before it — and specifically the shape of it: `keyword_value_is_foreign(property, &decl.value)` is computed **three lines below** the push it should be gating, and used only to write a diagnostic. The code identifies the exact declaration it must drop, says so out loud in a log message containing the word "dropped", and then keeps it. Three separate places in the tree assert the correct behaviour in prose (`stylesheet_compiler.rs:336-338`, `restyle_phase.rs:155-157`, `collect_invalid_value_faults.md:7`), a fourth (`keyword_value_is_foreign.md`) restates it, and none of them is true.

That is the pattern this whole pass kept hitting, and it is why a static sweep could not find any of it. In seven of the items above, the `.doc/*.md` file sitting next to the defect states the opposite of what the code does: `render_to_image.md` promises premultiplied bytes that are straight (B2); `MAX_DEPTH.md` promises it "never truncates a well-formed tree" while truncating one (B7); `positioned_run.md` promises "advances … untouched" while the advance is the one thing lost (B3); `narrow_f32.md` calls a bare `as f32` a "checked-narrowing point" (B4); `resolve_absolute_box.md` grounds itself in the CSS section it violates (A5); `initial_value.rs:205-224` asserts a fallback equals the CSS initial where it does not (A6); five `pointer_button_event.md` files justify a hand-rolled event with a `PointerConverter` limitation that no longer exists (A10).

Documentation lives outside the code here by house rule, and it is the primary artifact a reader — or a reviewing agent — trusts. It has drifted into being confidently wrong at precisely the points where the code is confidently wrong, so reading harder would not have found any of this. Only running it did.

---

## Pass 3 — one fact, spelled twice

Two spellings of one fact, drifted apart. Every item below names both sites, says which is wrong, and says what deletion or repair closes the pair. Findings are ordered by who pays: an application author or a user first, a maintainer last. In this pass a false `.doc` file counts as an author-facing cost, because under the house rule it is the artifact they read.

Twenty-four items. Two claims were dropped after adversarial verification (§ *Dropped*), five were materially narrowed, and the corrections are folded in below rather than reported as originally filed.

---

### 1. A pill button paints one shape and hit-tests another: 66% of it is dead

- **A** `crates/application/obscur-runtime/src/orchestration/scene_coordinator.rs:661` — `corner_radii_of(computed, node, Some(bounds)).normalized(bounds)`; the correction is `crates/foundation/obscur-geometry/src/primitive/corner_radii.rs:158-181`, the single CSS Backgrounds 3 §5.5 factor `f = min(side_extent / that side's two radii)` scaled across all four corners. `scene_coordinator.rs:645-652` states these are the radii every consumer sees.
- **B** `crates/application/obscur-runtime/src/runtime/hit_testing.rs:346` resolves the four corners with no correction; `hit_testing.rs:382-392` `clamp_pair` then clamps each radius independently to `rx.min(width)` / `ry.min(height)`. With a radius at or above the box dimensions every corner's quadrant test covers the whole box, so the hit region collapses to the intersection of four full-box ellipses.

**Wrong: B.** Executed — paint rasterized through `CpuRenderer`, hit side swept at 1 px through `UiRuntime::handle_pointer`:

```
pill 999px on 120x40    painted 4424  hittable 1512  dead 2912 (65.8%)  false-positives 0
uikit .btn-filled       painted 4024  hittable 1572  dead 2452 (60.9%)   (material_theme.rs:48)
uikit .nav-item         painted 8364  hittable 5404  dead 2960 (35.4%)   (material_theme.rs:180)
24px on 140x36          painted 4744  hittable 4544  dead  200 ( 4.2%)
circle 50px/100x100     dead 0 (0.0%)  [control, f == 1]
fits 40px/200x100       dead 0 (0.0%)  [control, f == 1]
half-height 20px/120x40 dead 0 (0.0%)  [control, f == 1 exactly]
```

Three `f == 1` controls agree to the pixel over ~31k samples, pinning the cause to `f < 1`. The error is one-directional: the hit test never accepts a point the paint rejects. Not deliberate — `hit_testing.doc/rounded_rect_contains.md:1-4` and `is_hittable.md:19-21` both promise this *is* the painted rounded border box, and `is_hittable.md:28-31` documents the file's *other*, intentional divergence at length. The repo's own uikit demo triggers it; the UA sheet does not, which is why CI is green.

**Done means:** `hit_testing.rs:346` resolves through `corner_radii_of(...).normalized(bounds)` and `clamp_pair` at `:382-392` is **deleted** — after normalization it is provably a no-op. Regression test: the pill's centre-left column at `y = top + 1` is hittable.

---

### 2. A control inside a transform is found by an inverse-transformed point and handed the raw screen point

- **A** `crates/application/obscur-runtime/src/runtime/hit_testing.rs:113-118` `local_point`, applied at `:149` and `:320` — the whole narrow phase runs in the node's untransformed space.
- **B** `crates/application/obscur-runtime/src/runtime/pointer_landing.rs:78-81` builds the delivered point from `HitTesting::content_point` (`hit_testing.rs:188-197`), which folds scroll only and applies **no** transform inverse; `runtime/ui_runtime/pointer_dispatch.rs:38` makes that `event.position` for every consumer. Measured against untransformed layout boxes by `slider_control.rs:63,69-79` and `text_pointer_selection.rs:113`.

**Wrong: B.** Executed (slider 116 px wide, 100 usable px, scale 0..100):

```
no transform (control)             press ( 58,12) -> 50.0   want 50   handler 1x
translate(100px,0) @ visual centre press (158,12) -> 100.0  want 50   handler 1x
translate(100px,0) @ UNtransformed press ( 58,12) -> —      handler 0x
scale(2) origin 0 0 @ visual centre press (116,24) -> 100.0  want 50   handler 1x
translate(-40px,0) @ visual centre press ( 18,12) -> 10.0   want 50   handler 1x
```

Row 3 is what makes this a pair rather than "input ignores transforms": targeting is fully transform-aware and refuses the vacated area. Row 5 nails the mechanism — `(18 − 8)/100 = 0.10`, the *screen* x read against the *unmoved* box. `pointer_landing.doc/folded.md` asserts the opposite of what the code does ("the space of the content the node it landed on holds"), and `resolve.md` enumerates exactly two corrections, neither a transform. `docs/unfinished.md:1225` cleared transformed hit testing — it verified targeting, which is the half that works.

Same shape, read not executed: `text_pointer_selection.rs:113` (caret placement), `pointer_dispatch.rs:431,439` (drag). Unproven and flagged as such: `landing.screen` (`pointer_dispatch.rs:37`) feeds pan/fling with no inverse either.

**Done means:** one function resolves the delivered point, sharing `local_point` with the hit test; `folded.md` and `resolve.md` describe it. Test: a slider under `translate(100px,0)` pressed at its visual centre reports 0.5.

---

### 3. Hit testing folds scroll in screen space; paint applies the scroll translate inside the CSS transform

- **A** paint nesting — `crates/application/obscur-runtime/src/orchestration/subtree_brackets.rs:180-196` (scroll bracket, `SCROLL_TRANSFORM_PUSH_ORDER = 1`) vs `:272-289` (CSS transform, `TRANSFORM_PUSH_ORDER = -3`; `orchestration/stacking.rs:31,40`). Emitted stream: `T ∘ clip ∘ translate(0,−scroll_top) ∘ box`.
- **B** `crates/application/obscur-runtime/src/runtime/hit_testing.rs:149` computes `local_point(node, folded_point(...))`; `folded_point` (`:178-185`) adds the fold to the **screen** y and `local_point` (`:113-118`) then inverts — i.e. `T⁻¹(screen + fold)`, the nesting `translate ∘ T ∘ box`. The correct inverse of A is `T⁻¹(screen) + fold`. Same asymmetry in the broad phase (`:65-66` vs `:222-229`) and in the ancestor clip test (`:312`).

**Wrong: B.** `scroll_top` is computed against untransformed layout boxes, so it is a local-space quantity, and `scroll_fold.md` says compose pushes the scroll transform *after* the container's own paint. Executed (`.port` 100×100 `overflow:auto`, four 100×50 rows, `scroll_by(port, 50)`):

```
scale(2) on the scroll container
  paint pushes: ["PushTransform([2,0,0,2,0,0])", "PushTransform([1,0,0,1,0,-50])"]
  (10, 10) hit-> r0   paint shows r1   MISMATCH
  (10, 60) hit-> r1   paint shows r1
  (10,110) hit-> r1   paint shows r2   MISMATCH
  (10,160) hit-> r2   paint shows r2
transform on an ANCESTOR      identical mismatches
no transform (control)        agrees at every point
translate(0,20px) (control)   agrees at every point
```

Hit resolves local `(y+50)/2`, paint `y/2+50` — a `|scale−1|·scroll_top` error, growing with both. Translations commute, which is why it survived. `hit_testing.doc/resolve.md` claims the opposite in writing.

**Done means:** the fold is added in local space at `:149`, `:312` and `:320` (`local_point` first), the broad-phase rect at `:66` carries the fold, and a hit-order test with `scale(2)` on a scrolled container asserts the click matches the painted row.

---

### 4. A focus ring paints outside its own element's transform and opacity layer

- **A** `crates/application/obscur-runtime/src/orchestration/subtree_brackets.rs:272-289` — transform bracket, popping at `transform_pop_order(depth) = i64::MAX − depth` (`stacking.rs:62-65`). `layer_pop_order` (`stacking.rs:57-59`) is the same schedule.
- **B** `subtree_brackets.rs:328-336` — outline unit at `OUTLINE_ORDER = i64::MAX` (`stacking.rs:46`), emitted last, geometry from the untransformed border box (`scene_coordinator.rs:285`). Sort key `(z, layer, tree_index, order)` at `scene_coordinator.rs:620`.

**Wrong: B**, and broader than "depth ≥ 1". Executed (`.btn` 80×30, `outline: 3px; outline-offset: 2px`):

```
control, no transform      1 body 0..80        2 ring -3.5..83.5      (agree)
translate(200px,0)         1 PushTransform  2 body  3 PopTransform  4 ring -3.5..83.5
opacity: 0.5               1 PushLayer(0.5) 2 body  3 PopLayer      4 ring at FULL alpha
DEPTH-0 root + transform   0 Push  1,2 bodies  3 Pop  4 ring        (ties, emitted last)
ancestor transform, .btn IS  last child   ... 4 PopTransform  5 ring OUTSIDE
ancestor transform, .btn NOT last child   ... 3 ring INSIDE  5 PopTransform
```

The last two rows are identical CSS giving two answers, differing only in whether an unrelated sibling follows. `stacking.doc/OUTLINE_ORDER.md` justifies the `i64::MAX` ceiling only against the node's **rounded clip** — the transform and layer escapes are unnamed anywhere. Live: `crates/components/obscur-theme-system/src/stylesheet/default_stylesheet.rs:23-26` ships `button:focus { outline }`, plus five more `:focus` rules.

**Done means:** the outline gets its own `outline_order(depth)` on the `i64::MAX − depth` schedule, emitted between the clip loop and the layer/transform loops — outside the rounded clip (the documented intent), inside the element's transform and group layer. No test in the tree covers outline together with transform or opacity.

---

### 5. A scroll container's scrollbar paints outside the container's transform, at the untransformed rect

- **A** `subtree_brackets.rs:272-289`, transform bracket popping at `i64::MAX − depth`.
- **B** `subtree_brackets.rs:345-358`, scrollbar unit at `SCROLLBAR_ORDER = i64::MAX` (`stacking.rs:43`), geometry from the raw `layout_box.bounds` (`scene_coordinator.rs:229-236` via `scroll_container_paint.rs:31-65`).

**Wrong: B**, same root as item 4 and to be fixed with it. Executed:

```
translate(500,300) on the container
  0 PushTransform  1 PushClip  2..6 bg + rows  7 PopClip  8 PopTransform
  9 track [92,0..100,100]  10 thumb          <- 500px from the box it belongs to
parent translated, container IS  last child   ...8 PopTransform  9 track   OUTSIDE
parent translated, container NOT last child   ...8 track 10 sibling 11 PopTransform  INSIDE
```

Escapes at every depth including 0 (tie broken by emission order), and escapes ancestor pops whenever the container is its ancestors' subtree exit node. The inline comment at `subtree_brackets.rs:343-344` — "Its rects lie within the scrollport, so being outside the clip is visually identical to being inside it" — is true of the `PopClip` the tier was chosen to clear and false of a `PopTransform` sharing that index. Contrast `SCROLLBAR_ORDER`'s scroll-transform escape, which *is* stated policy at `:339-346`. Bounded today only because the bar is not draggable (ledger item 23).

**Done means:** one tier strictly between the scroll translate's pop and the CSS transform's pop, shared with the outline fix.

---

### 6. Paint clips a subtree to the ancestor's rounded border box; the hit test clips it to a plain rectangle — and each side is wrong in a different direction

- **A** `crates/application/obscur-runtime/src/orchestration/scene_coordinator.rs:209-217` — gate `has_radius || clips_overflow`, shape `Clip::rounded_rect(bounds, radii)`.
- **B** `crates/application/obscur-runtime/src/runtime/hit_testing.rs:308-315` — gate `Self::overflow_clips(computed, parent)` only, shape `b.bounds.contains(...)`.

Executed (`.card` 100×100 green, `.chip` 30×30 red at its top-left corner; painted = measured pixel from `CpuRenderer`):

```
radius 40, overflow: visible   (3,3) white / nothing painted   hit-> chip
                               (8,8) white / nothing painted   hit-> chip
radius 40 + overflow: hidden   (3,3) white / nothing painted   hit-> chip
overflow: hidden, no radius    (3,3) chip painted red          hit-> chip   (control: agree)
```

**Two wrong sides, and which one flips with `overflow`.** (i) With a radius **and** clipping overflow, **B** is wrong: css-backgrounds-3 §5.3 puts overflow clipping on the border curve; on a 40 px radius the pointer accepts a point ~16.6 px outside the painted shape along the corner diagonal (~1373 px² of dead area live to a clipped-away child). `is_hittable.md` defends a 1.5 px border-width disagreement at length and is silent about this one, while its own Shape clause requires radii for the node itself (`hit_testing.rs:318-323`). (ii) With a bare `border-radius` and `overflow: visible`, **A** is wrong: CSS clips the element's own background and border to the curve, not a descendant's overflow. That behaviour is frozen by an internal unit test, `scene_coordinator.rs:3887`, whose fixture authors no `overflow`.

`hit_testing.doc/overflow_clips.md` claims the helper is "mirroring the scene coordinator's paint clip" — it mirrors only the second disjunct of the coordinator's gate. `docs/unfinished.md` item 18's "The four-way agreement is deliberate and documented, and it must be kept" is false the moment a radius is authored.

**Done means:** one `clip_shape_of(computed, boxes, node)` feeds `scene_coordinator.rs:212`, `hit_testing.rs:310`, `scroll.rs:101` and `scene_coordinator.rs:237`; the subtree clip is gated on `clips_overflow` alone with the node's own rounded fill clipped separately; `spec_ch4b_hit_testing.rs` gains a case for a child at the corner of a rounded `overflow: hidden` parent, and the `overflow: visible` counterpart. Fold into item 18's `padding_box_of` change — the reduced-radii clip and the rounded ancestor test are the same two numbers.

---

### 7. `text-overflow: clip` deletes every glyph when none fits, and the emptied box then re-enters the guard that refuses to clip a zero-width box

- **A** `crates/domain/obscur-text/src/layout/line_truncation.rs:38-56` (`glyphs.truncate(kept)`; with `kept == 0` the line is emptied), matched by `line_truncation.doc/truncate_line.md:4-6`.
- **B** `crates/domain/obscur-text/src/layout/paragraph_layout.rs:348-350` — `overflow.is_truncating() && bounded && max_width > 0.0`, commented "a genuinely 0-width box wraps to min-content (above) but is not clipped to nothing."

The seam: `orchestration/text_leaf_measure.rs:186` returns the paragraph's own size, so A's emptied line makes the leaf measure 0 wide; `orchestration/text_painter.rs:197-211` then wraps against that 0 and B refuses to truncate. Executed (font-size 40 ⇒ 24 px/glyph, 11 glyphs = 264 px, `white-space: nowrap`):

```
        domain unit (clip)   runtime (clip)   runtime (ellipsis)
w=0     11 / 264.0           11 / 264.0       11 / 264.0
w=5      0 /   0.0           11 / 264.0        1 /  24.0
w=10     0 /   0.0           11 / 264.0        1 /  24.0
w=23.9   0 /   0.0           11 / 264.0        1 /  24.0
w=24     1 /  24.0            1 /  24.0        1 /  24.0
w=100    4 /  96.0            4 /  96.0        4 /  96.0
```

Mechanism established by elimination: no positive `max_width` yields 11 glyphs except `≥ 264`, so the paint pass wrapped against **0**. Ellipsis keeps a carrier glyph through `kept == 0`, never collapses, and clips normally — that asymmetry is the tell.

**Wrong: A.** (i) It makes the leaf's intrinsic width non-monotonic — measure(0)=264, measure(5)=0, measure(24)=24 — a hazard for every shrink-to-fit / flex-basis / min-content consumer. (ii) `text_overflow.doc/TextOverflow.md` already describes B's model ("the layout records the intent, and a downstream painter enforces it"). (iii) CSS never removes content for `clip`. Live on the **default** path: `text_attributes.rs:94-105` returns `Clip` when `text-overflow` is absent, so a plain narrow column reproduces it with nothing declared.

**A third spelling, already false today:** `paragraph_layout.rs:1532-1534` — "(Was `0.0`, which is now a finite 0-width box **that clips**.)" It does not; `:350` excludes it.

**Done means:** the `kept == 0` branch at `:56` is deleted, `truncate_line.md:4-6` and the guard at `:350` say one thing, the stale comment at `:1533-1534` goes, and a test pins the band `0 < width < first-glyph-advance`. No test currently covers a positive sub-glyph width.

---

### 8. A spec-derived §4 test named `overflow_hidden_clips_to_the_padding_box` uses a border-less fixture

- **A** `crates/facade/obscur-kit/tests/spec_ch4_visual.rs:126` — fixture `.a { overflow:hidden; width:100px; padding:20px; height:100px }`, **no border**. With zero border the padding box and border box are the same rect, so the assertion cannot discriminate.
- **B** `crates/facade/obscur-kit/tests/spec_ch4_visual.rs:274` — same file, same distinction, fixture declares `border-*-width: 10px` on all four sides and asserts origin (10,10), 140×140. That fixture works.

**Wrong: A, and the code.** `scene_coordinator.rs:213-215` pushes `Clip::rect(layout_box.bounds)` — the border box. Executed: adding `border:10px solid rgb(0,0,0)` to A's fixture, changing nothing else, turns a green file red — `clip = origin (0,0) size 160×160` where CSS requires (10,10) 140×140.

`docs/unfinished.md` item 18 records the code defect and asks for "a test [that] asserts a child does not paint over a 10 px border" — apparently unaware that a spec test already carries that name and is green. This is the test half of item 18, and new.

**Done means:** A's fixture grows a border, and it fails until item 18 lands.

---

### 9. `readonly` is reserved in `obscur-view` and missing from `name_binding`'s mirror

- **A** `crates/domain/obscur-view/src/view/style_attribute.rs:26-27` reserves `READONLY = 0x0000_0007`; read at `:101-104` and by two live consumers — `runtime/structural_states.rs:27-28` (the mutation gate, via `ui_runtime/text_editing.rs:102`) and `runtime/semantic_builder.rs:34` (→ `node_converter.rs:57` `set_read_only`). Sole producer: `obscur-widgets/src/input/input.rs:222-223`.
- **B** `crates/frontend/obscur-ui-ir/src/name_binding.rs:348-376` — the reserved table maps eleven names and falls through at `:376` to `fallback_symbol`. Its own header at `:7` says the table mirrors `obscur_view::view::style_attribute`; `:363-370` argues at length why `placeholder` had to be reserved — verbatim the situation `readonly` is in.

**Wrong: B.** Executed end to end through a real `UiRuntime` with builtins and the UA sheet:

```
view::READONLY = InternedSymbol(7)   attr_symbol("readonly") = InternedSymbol(3456888823) = fallback
typed 'xy' into authored <input readonly="true">  -> Some("xy")   edited
typed 'xy' into authored <input>                  -> Some("xy")   control
typed 'xy' into Input::new(..).readonly(true)     -> Some("")     refused
typed 'xy' into Input::new(..)                    -> Some("xy")   control
(disabled, checked, placeholder, class, style, src, type, id all round-trip correctly)
```

Not deliberate: `okui-compiler/src/validation/builtin_contracts.rs:47` puts `.allowing_prop("readonly")` on the `Input` contract and `compile.rs:440` pins `<input readonly="true"/>` as compiling clean. Every comparable exclusion in `name_binding.rs` carries a written reason (`VALUE` at `:622-627`, asserted at `builtin_components.rs:618-623`); this one carries none anywhere in `crates/` or `docs/`. Costs the field both its write protection and its correct AT announcement.

**A third spelling:** `docs/api/09-authoring-formats.md:170-186` says "**Eleven** attribute names are reserved" and tables exactly B's eleven.

**Done means:** the arm in `name_binding.rs:348-376`, the constant beside `PLACEHOLDER` at `:21-22`, and the "Eleven" table all move together, and `crates/facade/obscur-kit/tests/okui_name_binding_drift.rs:25-45` — which exists for this class and names eleven symbols, neither `PLACEHOLDER` nor `READONLY` — asserts over `readonly`.

---

### 10. Four motion docs describe a three-kind `AnimatedKind`; the code has four, and one denies that a colour animates

- **A**, in descending severity: `crates/application/obscur-runtime/src/runtime/motion_driver.doc/animated_kind_for.md:5-7` — "A property outside this set (e.g. a colour or keyword) is not scalar-animatable here and drives no CSS transition/animation." Then `orchestration/animation_binding.doc/AnimatedKind.md:1,4-12` (public rustdoc, enumerates three as "this enum"), `animation_binding.doc/lower.md:3-4`, `motion_driver.doc/motion_value_of.md:5-10`.
- **B** `runtime/motion_driver.rs:630-636` maps seven properties (`Color`, `BackgroundColor`, the four border colours, `OutlineColor`) to `AnimatedKind::Color`; `:675-677` extracts it; `orchestration/animation_binding.rs:18-19,34-36` declares and lowers it; `crates/domain/obscur-motion/src/value/motion_value.rs:21-22,107-111,149` interpolates it premultiplied per css-color-4 §13.3.

**Wrong: A.** Executed through the facade: `transition: background-color 1s linear`, hover — 1 transition carried, 1 active animation, samples `Color(0,0,0) → (0.25,…) → (0.5,…) → (0.75,…) → (1,1,1)`.

Scope correction: `animated_kind_for` and `motion_value_of` are private items (`runtime/mod.rs:201`, `lib.rs:5`), so the affirmative denial reaches a reader of the tree, not published rustdoc. The two public files merely omit — and `animation_binding.doc/Color.md` documents the variant correctly one screen below the enum doc that excludes it, so the rendered page contradicts itself.

Why nothing caught it: `grep -rn 'AnimatedKind::Color'` across `crates/`, `checks/`, `examples/` returns only the two definition sites. `git log -S` puts the arm and the stale prose in the same commit (`e0792634`).

**Done means:** four prose edits, no code change; and a test pins the colour path so the four cannot drift again.

---

### 11. The CPU backend reports `BlendMode` as `Unavailable` in four places and composites all twelve modes to the byte

- **A** `crates/adapters/obscur-render-cpu/src/renderer/cpu_renderer.rs:453`; `cpu_renderer.doc/supports_effect.md:41-42`; `cpu_renderer.doc/_module.md:42`; and the unit test `cpu_renderer.rs:774-802`, which asserts "dropped, not approximated". A fifth spelling in `docs/conformance-audit.md:303` records "CPU had it right (`Unavailable`) throughout".
- **B** `cpu_renderer.rs:334-336` (offscreen when `blend != SourceOver`), `:365`/`:423` → `group_compositing.rs:20-33`, `skia_lowering.rs:9-24` (all twelve mapped). CSS reaches it as the layer *field*: `orchestration/composite_effects.rs:53` → `subtree_brackets.rs:254`.

**Wrong: A.** Executed, backdrop rgb(204,102,51) over layer content rgb(128,128,128):

```
Normal (128,128,128)  Multiply (102,51,26)  Screen (230,179,153)  Overlay (204,102,51)
Darken (128,102,51)   Lighten (204,128,128) ColorDodge (255,205,102) ColorBurn (153,0,0)
HardLight (204,103,52) SoftLight (204,102,51) Difference (76,26,77) Exclusion (127,128,128)
control: a layer transform IS dropped;  a blurred shadow DOES raise "unsupported effect: blur"
```

Every non-`Normal` value matches the separable-blend formula. The settling evidence is that the spec-derived oracle `sc_06_mix_blend_mode_multiply_exists` (`crates/facade/obscur-kit/tests/spec_ch5_paint_order.rs:176-207`) rasterizes through this very backend and asserts `rgb(0,0,0)` for cyan × red — it demands the reproduction and passes. `c9c9180f` added the lowering and moved neither the table, the two `.md` files, nor the test.

Two corrections to the original claim: the false `render.effect.unsupported` warning is raised **once per renderer instance**, not per frame (`effect_report.rs:159-162` dedupes with no reset path); and the *list* form `LayerEffect::Blend` genuinely is dropped, exactly as on Vello — which is why the right answer is `Limited`, not `Native`. Vello answers `Limited` for the same variant (`vello_renderer.rs:512-519`), so a host asking the capability model which backend does blend modes is told it backwards.

**Done means:** the arm becomes `Limited` with the split named (field honoured, effects-list form dropped), the two `.md` files, the unit test at `:774-802` and `conformance-audit.md:303` say the same thing, and the two backends' tables agree.

---

### 12. `LineBreaker` is documented as an installable runtime port and is a hardcoded local at both call sites

- **A** `crates/domain/obscur-text/src/layout/line_break.doc/LineBreaker.md:1-21` — "the exact analogue of the `Hyphenator` port"; "a host may install a tailored adapter"; "deliberately object-safe so the runtime can hold a `Box<dyn LineBreaker>`". Also `DefaultLineBreaker.md:9`.
- **B** `crates/application/obscur-runtime/src/orchestration/text_leaf_measure.rs:109` and `orchestration/paragraph_shaping.rs:191`, both `let line_breaker = DefaultLineBreaker;`. There is no `set_line_breaker` anywhere; contrast `runtime/ui_runtime/text_ports.rs:30` `set_hyphenator`, plus the hyphenator threaded in as a parameter at `text_leaf_measure.rs:38`.

**Wrong: B.** Proved by method resolution rather than grep: an extension trait declaring both setters printed its fallback body for `set_line_breaker` and not for `set_hyphenator`, and rustc corroborated with `method 'set_hyphenator' is never used` and no such warning for the other. The domain seam is genuinely live — `DefaultLineBreaker` gives 1 line for `"aaa-bbb-ccc"` at 24 px, a UAX #14-ish breaker gives 3.

**A third spelling:** `runtime/ui_runtime/text_ports.doc/_module.md:1-6` enumerates "four of the five" ports and silently omits this one.

Correction to the original claim: `ParagraphLayout::wrap_strut_hyphenated` is public and reachable via `obscur_kit::text`, so a host *can* pass a breaker to the domain API — what is unreachable is the toolkit's own frame path. Cost stands: Thai, Lao, Khmer and Burmese have no spaces and no ideograph pairs, so a paragraph is one unbreakable word, on the stated tablet target. `docs/conformance-audit.md:493` already records this gap; it is not in `docs/unfinished.md`.

**Done means:** either `set_line_breaker` lands beside `set_hyphenator` (threaded as a parameter, invalidating the shaped caches for the same reason), or `LineBreaker.md`, `DefaultLineBreaker.md` and `text_ports.doc/_module.md` say the port is a domain-API seam and deliberately not installable.

---

### 13. The only real `Hyphenator` adapter is a dependency of nothing

- **A** `crates/adapters/obscur-text-hyphenation/src/lib.doc/_module.md:10-13` ("the trait object the runtime installs via `set_hyphenator`"), `runtime/ui_runtime/text_ports.doc/set_hyphenator.md` ("A host installs a real adapter (e.g. Knuth-Liang English patterns) here"), and `Cargo.toml:119`'s `[workspace.dependencies]` entry.
- **B** No crate depends on it. `grep -rn obscur-text-hyphenation --include=Cargo.toml .` matches only the workspace root and the adapter's own manifest; the facade re-exports 18 adapters and not this one; there is no `hyphens` CSS property either.

**Wrong: B**, on the reading that the adapter ships — the facade is the only supported entry point (`checks/facade-consumer` exists to enforce that), so an adapter it does not re-export is unreachable by construction, including under `--features full`. `set_hyphenator` is a public installer with nothing in the workspace to pass it, and the `None` arm routes silently through `NullHyphenator`.

**Done means:** a `[dependencies]` line, a facade feature and a re-export — or the three doc files say hyphenation is deferred and the `[workspace.dependencies]` entry goes.

---

### 14. The `okui!` macro skips the capability rule the file front end enforces, and the doc says both run it

- **A** `docs/api/09-authoring-formats.md:463` — "Both front ends run the same rules from the same registry", with the `:479` capabilities row carrying no scope qualifier although the adjacent handler-names row says "file path only".
- **B** `crates/frontend/obscur-ui-macros/src/lib.rs:17-38` and `dsl/dsl_macro.rs:26-41` construct no `CapabilityValidator`; `crates/frontend/obscur-okui-compiler/src/compile.rs:272-275` does.

**Wrong: A**, the doc. Executed: `OkuiScreen::compile("<div requires=\"camera\">…")` → refused, `["required capability 'camera' is not granted at offset 0"]`; `okui! { <div requires="camera"/> }` → compiles, no diagnostic.

Two legs of the original claim are **refuted** and dropped: both front ends land `requires` as an ordinary hashed prop (`ui_ir_lowering.rs:190-203` falls through to `with_property`, exactly as `dsl_macro.rs:134-143` does), so that is not a divergence — but it makes `docs/api/09-authoring-formats.md:224-227`'s "structural rather than properties" false for `requires` on both paths, true only for `key`. And `Compiler::granting` *is* reachable through the facade and works from an out-of-workspace crate; what cannot grant is the **runtime** route — `crates/facade/obscur-kit/src/screen/okui_screen.rs:49-52` builds its `Compiler` with `defining(...)` only and exposes no override, so any `.okui` file declaring a capability is unloadable through `OkuiScreen`/`App`.

Also inert, recorded for completeness: the macro never calls `.requiring(...)`, so its document's `required_capabilities()` is empty where the file path's is populated.

**Done means:** the macro runs the same validator, or `:463` and the `:479` row say "file path only"; `OkuiScreen`/`App` gain a grant surface or `:479` records that the gate cannot be satisfied at runtime; and `:224-227` stops calling `requires` structural.

---

### 15. Four docs state `decode(encode(doc)) == doc` is the codec's contract; the format drops `key` and `source_map` and refuses two node kinds it emits

- **A** `crates/frontend/obscur-ui-ir/src/codec/ui_ir_encoder.doc/_module.md:27-29`; `codec/ui_ir_decoder.doc/_module.md:3-8`; `codec/mod.doc/_module.md:5-6`; `document/ui_ir_node.doc/UiIrNode.md:4-10`.
- **B** `codec/ui_ir_encoder.rs:99-124` never writes `node.key()` (`document/ui_ir_node.rs:46`, inside the derived `PartialEq` at `:40`); `codec/ui_ir_encoder.rs:23-40` never writes `doc.source_map()` (`ui_ir_document.rs:13`); `codec/ui_ir_decoder.rs:140` returns `UnsupportedNodeKind` for kinds 4 and 5, which `kind_tag` (`ui_ir_encoder.rs:84-95`) faithfully emits.

**Wrong: B.** Executed: `Some(Binding("row.id"))` → `None`, equality false; source map len 1 → 0; a `<For>` anywhere turns the whole document into `Err(UnsupportedNodeKind(5))`.

The control-flow refusal *looks* deliberate and is not. `ui_ir_decoder.doc/_module.md:15-22` justifies it with "[`UiIrNode`] currently exposes no public constructor for those kinds. Until such constructors exist…" — but `UiIrNode::loop_node` (`ui_ir_node.rs:77-80`) and `UiIrNode::conditional` (`:84-87`) are public and were called directly by the probe. The sentence landed in `24502d8c` (2026-07-04); the constructors landed in `5bfc4bf2` (2026-07-05). The named blocker has stood open for eight weeks.

Nothing fails in-tree — the codec's only callers are three tests in `spec_ch2_lifecycle_ir.rs`, none using a key, a source map or a control-flow node — but it is public API via `obscur_kit::ui_ir::codec` and the toolkit's own compiler emits exactly what it cannot carry (`ui_ir_lowering.rs:139,164,95`). `key` is reconciler identity: `bridge/ui_ir_view_bridge.rs:28,96` and `codegen/view_codegen.rs:389,425` lower it to `ViewNode::with_key`.

**Done means:** `key` and kinds 4/5 round-trip and the format spec lists them; `source_map` is either encoded or removed from the equality claim; the stale precondition at `ui_ir_decoder.doc/_module.md:15-22` goes; and `spec_ch2_lifecycle_ir.rs:291-307` asserts `== doc` on a keyed document containing a `<For>`, instead of comparing one `tag()`.

---

### 16. `truncate_line` states a postcondition it does not hold

- **A** `crates/domain/obscur-text/src/layout/line_truncation.doc/truncate_line.md:12-13` — "A partially-over glyph is dropped, so the resulting line width never exceeds `max_width`." Unqualified, and the same paragraph at `:11` explicitly contemplates the offending branch.
- **B** `line_truncation.rs:36` — `limit = max_width − ellipsis.x_advance()` goes negative; `:39-48` leaves `kept = 0`; `:52-55` still takes a carrier; `:57-65` pushes the ellipsis unconditionally with no comparison after.

**Wrong: A.** Executed, boundary exactly at `max_width < ellipsis.x_advance()`:

```
ellipsis-10    max_width=10    width=24.0  EXCEEDS
ellipsis-23.9  max_width=23.9  width=24.0  EXCEEDS
ellipsis-24.0  max_width=24    width=24.0  ok
clip-10        max_width=10    width=-0.0  (see item 7)
```

Cost narrowed by verification: the measure pass does **not** trust the invariant — `orchestration/text_leaf_measure.rs:170-186` ends in `constrain_allowing_overflow` with a comment saying a plain `constrain` "looks defensive and is a lie" — and min-content offers width 0, where truncation is switched off. Residual: a false stated postcondition on the artifact readers trust, plus a text leaf under a small positive width reporting 24 instead of ≤ 10, inflating a parent's content size. The three in-tree tests (`paragraph_layout.rs:1471,1487,1503`) only exercise cases where the ellipsis fits.

**Done means:** `truncate_line.md` states the exception (ellipsis wider than the box ⇒ line is exactly `ellipsis.x_advance()`, matching browsers) and says callers must not treat `<= max_width` as an invariant. No code change.

---

### 17. DevTools prints the Rust `Debug` identifier where two name tables already exist in scope

- **A** `crates/domain/obscur-style/src/property/property_id.rs:309` `PropertyId::name()` — the CSS spelling the parser accepts (`longhand_catalog.rs:146` matches on it); its doc says it is "intended for diagnostics and `DevTools` only". **A2** `property/layout_keyword.rs:583` `name_of` maps `0x1002 → "flex"`, `0x1A02 → "relative"`.
- **B** `crates/application/obscur-runtime/src/runtime/tree_inspector.rs:262` — `.map(|value| (format!("{property:?}"), format!("{value:?}")))`, in a file already importing `PropertyId` at line 5. Rendered as `{name}={value}` by `orchestration/inspect.rs:151` and `crates/facade/obscur-kit/src/devtools.rs:86`.

**Wrong: B.** Executed:

```
author wrote:  background-color: #ff8800;  display: flex
DevTools said: BackgroundColor=ComputedValue(Color(StyleColor { r: 1.0, … }))
               Display=ComputedValue(Keyword(InternedSymbol(4098)))
round trip:  .p { background-color: #ff8800 } -> 1 declaration
             .p { BackgroundColor: #ff8800 }  -> 0 declarations
```

The tool reports a property under a name its own CSS front end rejects and a keyword as a raw interner integer whose decoder is `layout_keyword.rs:583` — a forward table and its unused inverse, twice, on one pair. Every `.doc` beside B says "(name, value)".

Blast radius: three assertions pin the wrong spelling — `crates/facade/obscur-kit/tests/inspect_overlay.rs:116`, `okui_base_elements.rs:289` (under a comment reading "A computed `background-color`" — the drift in miniature) and `:297` (asserting on `StyleColor`'s `Debug` internals). None is a `spec_ch*` oracle.

**Done means:** `property.name()` on the property side and `name_of` for keyword values (both S, imports present); the three tests move. A full `ComputedValue` CSS serializer — the natural inverse of `property_id_from_name` — is a separate owner decision.

---

### 18. `LinuxDialogProvider::supports` answers `Native` for a request whose filters it drops

- **A** `crates/domain/obscur-system/src/dialog/dialog_request.rs:52,61` carries `filters: Vec<FileFilter>`; `dialog_provider.doc/supports.md` — "Reports the honest `SupportLevel` at which this provider can satisfy `request`."
- **B** `crates/adapters/obscur-system-linux/src/provider/linux_dialog_provider.rs:112` discards the request and answers `Native` whenever zenity is on PATH; `:124-133` never reads `filters`; `:249-254` pins the drop in a test ("Filters do not change the argv in this minimal mapping").

**Wrong: B.** zenity has `--file-filter`, so this is a fillable gap, not a platform limit — and even leaving the argv alone, the honest level for a request carrying non-empty filters is `Limited`, by the rule the sibling `Renderer` port states in writing. The contrast that makes it a defect: `WebDialogProvider` also cannot filter and reports `Unsupported` / `Unavailable` rather than claiming `Native`.

**Done means:** either `build_args` emits `--file-filter`, or `supports` returns `Limited` for a filtered request. No in-tree caller presents a dialog today.

---

### 19. `RenderTarget`'s format and row stride are declared, documented as contract, and read by nobody

- **A** `crates/domain/obscur-render/src/target/bitmap_target.rs:19-23,40,47,74-79`; `target/render_target.rs:7-11` — "the format chosen here is part of that precision contract"; same `with_format` on `window_target.rs:47` and `texture_target.rs:41`.
- **B** `crates/adapters/obscur-render-cpu/src/renderer/cpu_renderer.rs:152` reads `target.size()` and nothing else in the whole file; `crates/adapters/obscur-render-vello/src/renderer/vello_renderer.rs:491` the same (its format comes from `surface.blit_format()` at `:300`). `effective_row_stride` and `row_stride` have zero hits outside `bitmap_target.rs`.

Executed, 4×4 opaque red through `CpuRenderer`:

```
RGBA8  + with_row_stride(64):  effective_row_stride 64, ACTUAL row bytes 16
BGRA8  + with_row_stride(64):  first 8 bytes [255,0,0,255,…]   (blue should be byte 0)
Rgba16Float (HDR), packed:     effective_row_stride 32, bytes_per_pixel 8, ACTUAL 16, len 64
```

An HDR target silently yields an 8-bit buffer though both backends declare `.hdr(false)`, and `RendererError::InvalidTarget` already exists to refuse it. Not deliberate: `texture_target.rs:22-24` labels the neighbouring `sampled` field "a hint to the backend, which a backend *may* use" — the author marks advisory fields when he means it.

Latent: every in-tree call site uses bare `BitmapTarget::new(size)`; the exposure is entirely to an external facade consumer, which is the class `checks/facade-consumer` exists for.

**Done means:** `render` rejects a target whose format or stride it cannot produce with the existing `InvalidTarget`, **or** the prose is narrowed to advisory the way `sampled` already is and `effective_row_stride()` is deleted rather than left describing a buffer nobody produces.

---

### 20. The `TextShaper` port promises glyphs for every input; the only real adapter returns an empty run and the mock is what keeps the promise

- **A** `crates/domain/obscur-text/src/shaping/text_shaper.doc/TextShaper.md:10-13` — "a run always yields glyphs for its input"; `shape.md` — "never an error". `MockTextShaper` (`text_shaper.rs:46-68`) upholds it exactly.
- **B** `crates/adapters/obscur-text-harfrust/src/shaper.rs:83,86,89,96` — four `return empty()` arms for a non-empty input (no font bytes, `FontRef::new` failure, `units_per_em <= 0`), documented at its own site in `shaper.doc/HarfrustShaper.md`.

**Wrong: A, most likely** — the adapter's behaviour is deliberate and `can_shape` (`shaper.rs:135`) honestly returns false in the same conditions. What is wrong is an unqualified port invariant that only the mock satisfies: a headless test measures a text box with width; the same code against a face whose bytes fail to load measures zero, with no diagnostic on either side. Low today because `kit_bootstrap.rs:370-387` always registers the bundled Noto; it bites a host installing its own faces via `text_shaper_without_font` (`kit_bootstrap.rs:316`).

**Done means:** `TextShaper.md` states the qualifier ("infallible given a loadable face; `can_shape` is the gate"), or the adapter emits a diagnostic on the bytes-rejected path.

---

### 21. Two `#[test]` names claim properties their bodies cannot exercise

**(a) `arch_14_a_serial_adapter_and_a_parallel_one_produce_identical_output`.**
- **A** `crates/facade/obscur-kit/tests/spec_ch1_support/harness.rs:485-493` — `ThreadedExecutor::run_blocking` = `thread::spawn(job).join()`, semantically identical to `InlineExecutor` at `:475-481`, the same test's "serial" arm.
- **B** `crates/adapters/obscur-task-smol/src/spawner/smol_spawner.rs:77-79` — `blocking::unblock(job).detach()`: the shipped adapter returns before the job runs, which is the whole content of §1.5.

Executed: replacing the "parallel" body with a bare `job();` leaves arch_14 green; replacing it with production semantics (return before completion) makes it fail, the second frame carrying three `DrawText` commands and **no** `DrawImage` at all. **Wrong: A.** Any defect requiring `run_blocking` to return early — the shipped adapter's only mode — is invisible to §1.5's oracle. (`arch_15` is sound: `QueuedExecutor` genuinely defers.)

**(b) `a_system_gesture_wins_the_arena_by_default`.**
- **A** `crates/facade/obscur-kit/tests/spec_ch6b_gestures.rs:20` — both contenders are priority 0; SYSTEM_BACK is simply added second.
- **B** `crates/domain/obscur-interaction/src/gesture/gesture_arena.rs:41-52` — `priority >= best_priority` in insertion order; the in-crate test at `:169` names it honestly, `tie_priority_breaks_to_most_recently_entered`. The arena has no system-gesture concept at all.

Executed: swapping only the two `add` calls — platform first, app second, the natural order — turns the test red. **Wrong: A.** Low today (item 7 records the arena has no production consumer); it costs the moment it is wired.

**Done means:** (a) the "parallel" harness returns before completion, or the test is renamed for what it checks; (b) either the arena gains a system tier and the contender is built with it, or the test is renamed to the tie-break rule the code implements.

---

### 22. §4.9's scroll-anchoring oracle drives a mechanism the runtime never calls

- **A** `crates/facade/obscur-kit/tests/spec_ch4b_virtualization.rs:77-94` — calls `anchor_at`, `estimator_mut().set_measured(10, 150.0)` and `anchor.resolve(...)` on a `VirtualLayout` it constructed.
- **B** `crates/application/obscur-runtime/src/runtime/virtual_lists.rs:210-213` and `:302-305` — the only production construction: `VirtualLayout::new(SizeEstimator::new(extent), OverscanPolicy::symmetric(VIRTUAL_OVERSCAN))`, rebuilt inside the realize closure every frame from one uniform extent and discarded at the end of it. `set_measured` and `anchor_at` have exactly one caller each in the tree, both in that test file.

**Wrong: B** (the runtime never uses the mechanism), **and A** for being named for it. A variable-height virtualized list — the toolkit's origin use case — positions every row at the uniform extent and lurches whenever a row above the viewport differs; the chapter that owns that is green. The same file's `ch4_9_a` drives a real `UiRuntime`, so the uniform green reads as end-to-end coverage.

Adjacent, cheaper: `ch4_2_c_custom_layout_is_deterministic_across_processes` (`spec_ch4b_layout_contract.rs:52`) spawns two processes to guard `CustomLayout::layout` (`crates/domain/obscur-layout/src/algorithm/custom_layout.rs:36-70`), a plain indexed loop with no map, set or address dependence — and `orchestration/formatting_context.rs:18-28` dispatches only Flow, Flex and Grid. The guard works, on code nothing reaches.

**Done means:** the estimator survives a frame and the anchor runs, or the test's `.doc` says it exercises the domain API only; and the determinism guard points at the Flow/Flex/Grid path where `FastMap` iteration actually occurs.

---

### 23. Two spellings of "scan a core crate's public signatures", and a wrapped `pub fn` in `obscur-view` is invisible to both

- **A** `crates/facade/obscur-kit/tests/spec_ch9_support/harness.rs:247` `public_api_lines` — iterates `text.lines()` and keeps lines starting with `pub `; a rustfmt-wrapped signature contributes only its `pub fn name(` line. Consumed by `spec_ch9_api_surface.rs:21,65`.
- **B** `crates/facade/obscur-kit/tests/spec_ch1_layering.rs:58` `public_signatures` — accumulates while paren depth > 0, reassembling the signature before the needle scan at `:92`.

**Wrong: A.** Executed, three runs:

```
1. single-line PathBuf leak in obscur-tree              -> §9 FAILS (3 leaks)
2. same leak wrapped over lines, obscur-tree            -> §9 ok, §1 arch_01 FAILS
3. same wrapped leak moved to obscur-view               -> BOTH GREEN
```

The crate lists also disagree: §1's `UPWARD_CRATES` (`spec_ch1_layering.rs:12`) covers geometry and motion but not `obscur-view`; §9's `CORE_CRATES` (`spec_ch9_api_surface.rs:10`) covers `obscur-view` but not geometry or motion — so `obscur-view` is scanned only by the broken scanner. A multi-line-aware sweep of all six `CORE_CRATES` today finds zero live leaks: this is a hole in the guard, not a breach.

**Done means:** one scanner (B's), one crate list, both oracles calling it — the layering invariant everything rests on is currently checked by a scanner that stops at column 90.

---

### 24. Three stale absence claims

**(a)** `crates/adapters/obscur-window-winit/src/backend/winit_window_backend.doc/native_window.md:10-12` — "every windowed example calls `event_loop.create_window` itself and this backend went a whole campaign with no caller." **B:** `crates/facade/obscur-kit/examples/windowed_counter.rs:101` creates through the port and `:250` reads the handle back; the file contains no `event_loop.create_window`. The correct spelling already exists one directory over and points *back* at the false one: `examples/windowed_counter.doc/_module.md:19-27` ("the window is created **through the toolkit's own §6 window port**"), with `:52` sending the reader to `native_window` "for why that handle is on the adapter". The clause was false when written (`f5ebdb83`, the very commit that added both) and was moved verbatim into the `.doc` file by `e0792634`.

**(b)** `crates/diagnostics/obscur-devtools-server/src/endpoint.doc/pending_controls.md:3-4` — "`drain_controls` has no caller in this workspace." **B:** `crates/facade/obscur-kit/tests/spec_ch9_devtools.rs:109`. The sibling `endpoint.doc/drain_controls.md:3` says it correctly ("No production code calls this"), which is also the house convention across a dozen other files. Executed from an out-of-workspace probe: handshake accepted, `dispatch Control(Pause) → Ok(Acknowledged)`, `drain_controls() → [Pause]`, second drain `[]` — not a stub.

**(c)** `crates/adapters/obscur-shell-layer/src/surface/layer_window.doc/supports.md:3` — "**Six of the seven**". **B:** `crates/domain/obscur-windowing/src/port/layer_backend.rs:11-40` — `LayerFeature` has eight variants and `ALL: [Self; 8]`; `layer_window.rs:347-360` lists seven unconditionally with `FractionalScale` conditional. `InputTransparency` was added and the prose kept its own copy of the count — which `layer_backend.doc/ALL.md` exists to prevent ("which is how a new feature comes to be silently unanswered"). Item 39 already schedules an edit to this file; fix it in the same touch.

**Done means:** three prose edits. All three are the Part-4 stale-ledger-row class, where a false "nothing uses this" is how work gets re-done or code gets deleted.

---

## The doc-drift rate

The doc lane sampled by two filters — falsifiable postcondition language (`never|always|guarantee|cannot|exactly|monotonic|idempotent`, 1,992 lines in 1,536 files), absence claims, and prose naming a second spelling it must agree with — restricted to hot modules, then read the code beside each claim and ran it where the claim was numeric or behavioural.

**32 checkable doc claims resolved. 25 agreed. Of the 7 that did not, 4 survived adversarial verification, 1 was refuted, and 2 are stale-by-omission in the same cluster.** Hard-false rate ≈ **6%**; drift-carrying ≈ **19%**.

The rate is not the story. Two things are:

1. **Concentration.** Four of the seven sit in one subsystem (CSS motion), where the code grew a fourth animated kind and none of its four prose files followed. That is a decomposition-pass omission with a commit hash (`e0792634`), not background rot.
2. **The doc was the wrong side in roughly half of every confirmed finding in this pass, including findings the doc lane never touched** — the CPU blend table and its two `.md` files and its unit test; the codec's four round-trip docs; `pointer_landing.doc/folded.md`; `hit_testing.doc/overflow_clips.md`; `LineBreaker.md`; the `readonly` "Eleven" table; three absence claims. Every one of them was written true and then outlived a code change that shipped in the same commit or the next day.

So: **a cleanup, with one systemic cause.** The `.doc` file is not being read as part of the change that invalidates it. Nothing enforces the pairing — there is no `xtask` rule that a touched `pub` item's `.doc` file must be touched too, and no test anywhere asserts a doc's claim. The four instances that are pure prose (items 10, 16, 24) cost one edit each; the value is in not manufacturing five more next month.

---

## What agreed

Fifty-two pairs were opened, both sides read, and found consistent. On a codebase where two spellings are the known failure mode, this list is worth as much as the defect list.

**Forward tables and their inverses — all whole.** `layout_keyword`: all 94 `pub const` have a `name_of` arm, every string `name_of` returns is accepted by some `*_from_name` producer, and the 7 constants with no producer are minted by `grid_placement.rs:176-194` / `z_index_from_name` instead. The `keyword_from_name`/`name_of` defect fixed on 2026-08-28 is genuinely closed. `PropertyId::name()` (122 arms) vs `longhand_catalog::ALL` (121): exactly one exclusion, `GridArea`, correctly — it is a shorthand marker only `shorthand_expansion.rs:413` mints. The devtools wire codec is symmetric at every discriminant (rejection 2, scope 4, inspectable 12, topic 4, severity 5, plus six inline matches); unknown tags error rather than defaulting. All 11 `PaintCommand` variants are matched by both backends; all 10 `FilterOp` variants by all three consumers; all 16 `RESERVED_EVENTS` have a runtime dispatch site.

**Identity namespaces — partitioned.** `InternedSymbol::named`/`from_index` both set bit 31, reserved constants leave it clear, and `name_binding::fallback_symbol` uses the identical FNV-1a so `attr_symbol("label") == InternedSymbol::named("label")`, guarded at `name_binding.rs:1078-1087`. `ComponentId` sets bit 63 for user components. This is the `CssSymbolInterner` class of defect, closed on both id types.

**The strongest maintained forward/inverse pair in the tree:** `property_value.doc/uses_ch.md` claims `absolutize_value` and `uses_ch` visit the same `Length`s. Compared `property_value.rs:83-128` against `absolutize.rs:11-75` arm by arm — identical arm sets, both exhaustive, no wildcard on either side — then descended into `RadialGradient` (`radial_gradient.rs:65-72` vs `absolutize.rs:169-181`) and `ConicGradient` (`conic_gradient.rs:55-57` vs `:184-191`), including both sides agreeing that gradient stop offsets carry no length. Likewise `logical_property.doc/is_logical_target.md` names a real guard: `logical_cascade.rs:103-127` iterates `PropertyId::LOGICAL × {rtl,ltr} × 3` writing modes, with a second test closing the other direction.

**Paint vs pointer, where it works.** `resolved_transform` (`paint_transform.rs:11`) is one function shared by `scene_coordinator.rs:244` and `hit_testing.rs:100`, so transform-origin cannot drift. `scrollbar_rects` (`scroll_container_paint.rs:47`) is one function shared by paint and the DevTools inspector (`tree_inspector.rs:171`). And the slider is the model: `THUMB_RADIUS = 8.0` (`slider_painter.rs:15`) and `THUMB_INSET = 8.0` (`slider_control.rs:12`) are the same number in the same formula, reading the same props through byte-identical helpers, with the two coordinate spaces held together by the UA sheet (`slider { }` deliberately bare; `progress > canvas, slider > canvas` sizing the painter box), both pinned by tests at `default_stylesheet.rs:453-472` that say why. It is the only paint/pointer pair in the tree actively defended by a test.

**Text measure vs text paint.** `measure_face()` (`text_leaf_measure.rs:316`) and `ParagraphShaping::default_face` (`paragraph_shaping.rs:53`) construct an identical `FontFace` field for field, and `default_face` has no setter. `measure_text_node` and `shape_lines` build the same `LineStrut`, the same `seg_metrics` closure, and call `wrap_strut_hyphenated` with the same arguments. The hyphenator is a parameter on both sides and `arrange_phase.rs:151` passes the same `scene_composer.hyphenator()` the compose pass uses — the classic measure/paint line-count mismatch does not occur here. `SelectionGeometry::from_paragraph` prefix-sums the advances `line_caret_x` filters, and in LTR the filtered set is exactly that prefix.

**Defaults spelled more than once, agreeing.** Initial font size 16.0 in five places (`initial_value.rs:7`, `layout_style.rs:92`, `text_style.rs:96`, `length_resolution.rs:14`, `paragraph_shaping.rs:29`) with every consumer's fallback matching. `FLING_TAU`/`TAU` = 0.325 in `fling.rs:58` and `pointer_sessions.rs:24`. `LayoutStyle::default` matches the CSS initial for every deferred keyword re-checked; `align-content` is the only counterexample and is already Pass-2 item A6. `MIN_TILE_EXTENT = 0.5` and `MAX_TILES_PER_AXIS = 8192` are identical across both render backends, applied identically, and their `.doc` files say the same thing in different words.

**Ports whose adapters agree.** Timer (smol vs tokio, identical semantics and near-identical docs). `CacheStore` (in-memory vs fs): `CacheKey` is `(u64, u64)` and the filename `"{resource}-{variant}.oc"` is injective over it; `ResourceVersion` has one field, so comparing the stamp *is* struct equality. `WindowBackend::supports` — every arm backed by the matching method, `Kiosk` refused outright rather than degraded silently. `AppearanceProvider` (portal/GTK/Qt/composite) — the most carefully reconciled multi-adapter port in the tree: `support()` and `watch_support()` deliberately independent, GTK `Limited` because it genuinely cannot state a palette, the composite merging field-by-field with the KDE-plus-portal case named in the comment. `ShellProvider` (four impls, `can_open` and `open` agreeing in all four). `TextItemizer` — both impls cover `0..text.len()` contiguously, the real one a strict refinement of the reference. `FontResolver` — the mock's coverage-blindness is stated policy in its own doc. Resource loaders/decoders: the metadata asymmetry between mocks and real adapters is measurably inert (no consumer branches on `mime_type`), and the one place a mismatch would bite — alpha convention — agrees (`AlphaMode::Straight` on both sides).

**Foundation invariants that hold as written.** `AffineTransform::inverse` really does test all six coefficients for finiteness after the reciprocal, not just `det != 0`. `map_rect` folds all four transformed corners. `TextStyle::new` clamps 1..=1000. `ScriptTag::as_str` is infallible by construction. `OverscanPolicy::expand`, `SizeEstimator::offset_of`, `MaterializedRange::new` and `VirtualLayout::visible_range` all clamp exactly as documented, including the off-by-one step-back at `virtual_layout.rs:50-53`. The hyphenator's non-interior-offset clamp (`break_offsets.rs:77-82`) is really there. `ScrollTargets`' whole thesis — `scrollable_at`/`enclosing_scrollable` filter on `overflow_scrolls` while `fold_point` must not — holds in all four methods, including the `hidden`-still-folds case its doc singles out.

**Duplicates that currently agree** (drift hazards, no live defect): `overflow_for_text` verbatim in `measure_pass.rs:70` and `paragraph_shaping.rs:456`; `tile_index_range` byte-identical in both backends; `split_layers` identical in `animation_compiler.rs:244` and `transition_compiler.rs:63`; `HitTesting::overflow_clips` and `scroll_container_paint::overflow_clips` the same one-liner; the entire keyframe sampler duplicated between `MotionCoordinator::sample_keyframes` (`motion_coordinator.rs:13-53`) and `KeyframeTrack::sample` (`keyframe_track.rs:133-171`), including both copies of `last_keyframe_at_or_before`.

**One latent hazard, not a finding.** `text_attributes::length_to_px` (`:126`) handles only Px/Em/Rem where `length_resolution::length_px_of` (`:100`) handles the full relative set plus `Calc` — so `letter-spacing`, `word-spacing`, `line-height` and `tab-size` would read 0 for units the rest of the box model resolves. It cannot bite today: `Length::absolutize` (`length.rs:75`) folds every relative unit to Px before storage, with the single exception of `ch` before `ch_width` is measured. One `ch` ordering change makes it live.

**Also checked and clean:** `rg should_panic` over the whole tree returns nothing, so the `#[should_panic]`-without-`expected` hazard does not exist here; every `#![cfg(...)]` at the head of a facade test file names only default features, so no file is silently skipped by CI; and a multi-line-aware scan of all six §9 `CORE_CRATES` found zero live platform-type leaks.

---

## Dropped

Two claims were dropped whole after adversarial verification, and five were materially narrowed.

**Dropped: `on:click="save"` in the drift guard's comment.** Executed through the real compiler: on a plain `<div>` the quoted spelling compiles clean and reaches exactly `InternedSymbol::named("save")`, so `name_binding.rs:1078-1086` — which speaks explicitly of an *unreserved* name on an ordinary element — is literally true, and `check_string_dispatch` (`contract_validator.rs:226-246`) only ever runs against a registered contract, as `named.md:16-17` says. A third opinion exists and reconciles them: `docs/api/09-authoring-formats.md:155-164` states both halves and calls the quoted form "the worst of the three outcomes". No path behaves two ways; no code disagrees with code.

**Dropped: three of five `AccessibilityBridge` methods have no production caller.** The facts hold — measured over an install push plus ten tree-changing, focus-moving frames: `update_tree` 11, `set_focus` 11, `apply_update` 0, `announce` 0, `poll_actions` 0, with a queued AT action stuck in the bridge. But each silent channel is a non-goal recorded with reasons: the inbound drain is §7.1 L27, named as unbuilt in `ui_runtime/accessibility.doc/set_accessibility_bridge.md` and `kit_bootstrap.doc/install_accessibility.md:34-38`; `announce` has no runtime caller because no live-region source exists anywhere in the semantic vocabulary (`docs/conformance-audit.md:433`, confirmed by grep); `apply_update` is an optional incremental channel the only adapter collapses to a dirty flag that the runtime's full-snapshot push already satisfies. And `is_dirty`/`take_announcements` having zero in-workspace readers is the same fact as `last_tree_update`/`push_action` having none — the platform half belongs to a windowed host the workspace does not contain (`install_accessibility.md:15-32`). Residue worth one sentence, not a finding: the `announce` gap is disclosed in `docs/` and in no `.doc` file.

**Also recorded, not findings:** the ports with zero or one implementation — `LocalSpawner` (no impl anywhere, not even a test double), `ResourceCompiler` (only `NullCompiler`), `HistoryProvider`, `DocumentService`, `LazyDataProvider`, `Clock`, `ChMetrics`. Each is either a deletion or a missing adapter and nobody has said which, but none is a pair of disagreeing spellings.

---

## The pair that should never have been two spellings

**`clamp_pair`, `crates/application/obscur-runtime/src/runtime/hit_testing.rs:382-392`.**

It is a second, private implementation of a rule the foundation crate already owns: `CornerRadii::normalized` (`crates/foundation/obscur-geometry/src/primitive/corner_radii.rs:158-181`) implements CSS Backgrounds 3 §5.5 correctly, is public, is tested — and one of those tests (`:475-486`) exists specifically to record that a per-corner clamp is the approach §5.5 replaced. The hit test wrote its own anyway, looser than the retired one, with a comment stating the §5.5 goal it does not reach.

The right fix is not to repair `clamp_pair` to match `normalized`. Two correct implementations of one rule is exactly how this pass's findings were manufactured. The fix is to resolve the hit radii through `corner_radii_of(...).normalized(bounds)` — at which point `clamp_pair` is provably a no-op, because normalized radii always satisfy `rx <= W` and `ry <= H` — and **delete** it. One spelling of §5.5 in the tree, in the crate that owns the geometry, with the paint path and the pointer path both calling it.

Runner-up, currently agreeing and therefore cheap to close before it does not: the keyframe sampler is duplicated line-for-line between `crates/application/obscur-runtime/src/orchestration/motion_coordinator.rs:13-53` and `crates/domain/obscur-motion/src/track/keyframe_track.rs:133-171`, helper included. One of the two should be deleted while they still agree.

---

## Pass 4 — time, state and failure

Three sweeps looked at the system at rest, on one frame, with everything working. This pass drove it: warm runtimes against cold ones over thousands of mutation sequences, long-running frame loops with churning trees, and every port broken on purpose. Five lanes, each working from an out-of-workspace probe crate on the `checks/facade-consumer` pattern (own `[workspace]`, one path dependency on `crates/facade/obscur-kit`), release builds, `CARGO_TARGET_DIR`/`TMPDIR` under `/var/tmp`, every build inside a memory-capped `systemd-run --user` unit. Adversaries then re-ran the load-bearing claims from independent probes. The repository was not modified by any of them.

The headline is two-sided and both sides matter. **The incremental style/layout/paint/hit pipeline computes the right answer** — 1800 random mutation sequences and 43 hand-built adversarial ones produced exactly one divergence from a cold render, and that one is a scroll offset, not a style. **It is only unbounded**: nothing on the incremental path drops the style state of a node the reconcile removed, so a list that scrolls leaks 114 MB per minute and degrades ~34× in frame time against the full pass it exists to replace.

Absolute millisecond figures below are machine-load-dependent (four agents shared this box; one adversary ran under load average 20). Ratios against a control on the same box in the same conditions, and RSS, are the load-independent numbers and are what the claims rest on. Both are given.

---

### Part A — the incremental path disagrees with a cold render, or leaves state nothing can repair

#### A1. The hover chain is only ever written by a pointer event, so `:hover` paints on the wrong element after any frame that moves, removes or scrolls a box

Three shapes, all release, all through the public API, all with the pointer stationary.

**A box grows under the cursor.** Sheet `.row{height:50px;background-color:#000} .row:hover{background-color:#f00} .tall{height:150px}`; three rows; move to (10,120) so row 2 is hovered; re-render with row 0 at 150 px.

```
[hover@120 (row2)] hovered() = Some(4294967300)  #r2 · 0,100 200x50
[after r0 grows]   hovered() = Some(4294967300)  #r2 · 0,200 200x50
    #r0 (0,150) bg=Color(0,0,0,1)   #r1 (150,50) bg=Color(0,0,0,1)   #r2 (200,50) bg=Color(1,0,0,1)
[re-move to the SAME point] hovered() = Some(4294967298)  #r0.row.tall · 0,0 200x150
```

Not just a computed field. Filtering the emitted `DisplayList` for red fills, with the cursor fixed at y=120: red at `(100,50)` before, `(200,50)` after the growth, `(0,150)` after a jiggle to identical coordinates. The `:hover` background is composed onto a row 80 px from the cursor, and the row the cursor is inside is left unstyled until the user moves the mouse.

**The hovered row is deleted** (5 rows → 3, hover at (10,220)): `hovered() = Some(4294967302)`, `still in tree = false`, and `hovered_inspect()` returns `None` so DevTools goes blind. The control is one line away in the same frame hook: the same removal leaves `focused = None`.

**A wheel scroll — the commonest trigger, and not in the original claim.** Ten 50 px rows in a 200 px `overflow-y: scroll` viewport, cursor at (10,120): after `handle_scroll(100.0, (10,120))` the list scrolls and `hovered()` still names `#r2`; a jiggle to the same point re-targets `#r4`.

`crates/application/obscur-runtime/src/runtime/ui_runtime/pointer_dispatch.rs:69` (`self.hover = chain;`) is the only writer of `UiRuntime::hover`; a grep of `self.hover` across `obscur-runtime` returns that write and five reads (`pseudo_state_projection.rs:46`, `frame_pipeline.rs:272`, `interaction_state.rs:14,:34`, `diagnostics.rs:66`). The reconciled-frame hook exists and covers only focus: `frame_pipeline.rs:134` `self.drop_focus_if_removed()`, whose own doc names the defect it closed — "`focused` went on naming a dead id". `handle_scroll` (`ui_runtime/scrolling.rs:110`) is not `handle_pointer`, so it never touches the chain either. Not deliberate: `ui_runtime.doc/hover.md` ("the nodes currently under the pointer, topmost-first") and `interaction_state.doc/hovered.md` assert the opposite, `hover_update.doc/for_event.md` records only the touch-release policy, and nothing locks in the current behaviour — the facade's `incremental_restyle.rs` drives hover through `set_pseudo_state`, bypassing the chain. `specs/w3c/selectors-4.txt` §9.1 lists `remove-hovered-element.html` among the `:hover` tests.

**Effort:** M. The removal half is the cheap one — a `drop_hover_if_removed()` beside `drop_focus_if_removed()`. The moved-box and scrolled-box halves need a re-hit-test after layout, and `PointerSessions` retains presses, pans, holds, captures and last clicks but no last hover position, so a last screen point per hover-capable pointer has to be recorded first. **Done means:** a test grows a row under a fixed cursor and asserts the red fill moves to the row containing the cursor in the same frame; a test deletes the hovered row and asserts `hovered() == None`; a test wheel-scrolls under a fixed cursor and asserts the new row hovers.

#### A2. A scroll offset outlives the box ceasing to be a scroll container, so re-showing it jumps the content 150 px with no gesture

One node, class flipped between `.scroller{height:50px;overflow:auto}` and `.plain{height:50px}` with the ComponentId held constant (so the reconciler updates in place), 10 × 20 px items inside.

```
A. scrolled scroller : node=NodeId(4294967297) scroll_top=150 hit(10,10)=item#8
B. same node, overflow gone : same_node=true scroll_top=0 hit(10,10)=item#0
B'. set_scroll_top(node, 0.0) while not a container -> ignored, no effect
C. scroller again    : same_node=true scroll_top=150 hit(10,10)=item#8
D. replaced by a NEW node : same=false scroll_top=0 hit(10,10)=item#0
E. COLD scroller     : scroll_top=0 hit(10,10)=item#0
K. display:none round trip, same node : 150 -> 0 -> 150
```

Scene stream, warm C vs cold E: warm len=6, cold len=4; warm carries `PushTransform(AffineTransform { coeffs: [1,0,0,1,0,-150] })` + `PopTransform` the cold render does not emit. This is the single divergence the differential harness flagged out of 43 hand-built scenarios (`scroll-container-destroyed-and-recreated`): 10 layout boxes off by exactly −150 px and ~100 hit-grid points naming a different row (warm `/0/7`, cold `/0/0`).

Two things this is not. The `overflow` flip is contrived; **`display: none` is the archetypal tab-panel / collapsible pattern and reaches the same ghost** — the box loses its layout box, drops out of `scroll_containers` (`orchestration/scroll.rs:96-98`, which needs both `establishes_scroll_container` and `boxes.get(&node)`), and the offset returns on re-show. And the recovery path is closed while it matters: `set_scroll_top` early-returns for a node that is not a live overflowing container (`ui_runtime/scrolling.rs:71-74`, documented), so an application cannot pre-emptively clear the ghost — only correct after the jump is painted. Note that `inspect_snapshot` reports content-space boxes (`orchestration/inspect.rs:110`), so `item0.y` reads `0.0` in every case; the −150 lives in the paint transform, the hit fold and `ScrollOffsets::projection`.

Two boundaries a fix must not cross, both measured. `overflow: hidden` correctly does **not** reset (150 / 150 / 150) — `establishes_scroll_container` (`overflow_axes.rs:86-94`) keeps `hidden` a container, matching CSSOM. And the content-shrink neighbour is stated policy through the identical code path: content 10 → 1 → 10 items with `overflow` never removed gives 150 / 0 / 150, which `scroll_offsets.doc/ScrollOffsets.md` blesses. `ScrollOffsets` deliberately holds "neither the tree, the boxes nor the computed styles", so it cannot tell the two apart — the fix belongs in the caller that can see box-presence. The docs do contradict the behaviour on this case: `scrolling.doc/scroll_top.md` ("an unscrollable box is at the top by definition") is falsified one frame later.

`crates/application/obscur-runtime/src/runtime/scroll_offsets.rs:23` — `offsets: FastMap<NodeId, f32>`, with no `remove`, `retain` or `clear` anywhere in the workspace; `runtime/layout_classifier.rs` mentions neither `Overflow` nor `Display`.

**Effort:** S. **Done means:** one decision, then either `ScrollOffsets.md` covers it explicitly, or a caller seats the offset to 0 when the node stops having a scroll container's box — scoped to `establishes_scroll_container == false` or "no layout box", leaving `overflow: hidden` and the content-shrink case byte-identical. A test asserts the `display:none` round trip ends at 0.

#### A3. One NaN frame delta into `advance_motion` hangs the UI thread if any animation repeats infinitely, and otherwise completes every timeline, fires every armed long press, and leaves an immortal NaN fling

**The hang is the ordinary CSS path.** `@keyframes spin{from{width:0px}to{width:100px}}` + `.row{animation:spin 1s linear infinite}` through `CssPipeline`, render, `advance_motion(0.25)`, then `advance_motion(f32::NAN)`:

```
t=0.25 active_animations=1
calling advance_motion(NaN) ...        <- last line printed
EXIT=124        # timeout 180, release
```

Same through `animate_with` with `MotionPlayback{repeat: Infinite}`, and on the bare `Timeline`. `animation-iteration-count: infinite` compiles to `RepeatCount::Infinite` at `crates/frontend/obscur-css/src/compiler/animation_compiler.rs:269`. A spinner is the most common CSS animation there is.

`crates/domain/obscur-motion/src/timeline/timeline.rs:142` is `if remaining <= 0.0 { return; }` — false for NaN. `remaining < time_to_end` is also false, so control falls into the "iteration completed" branch, `remaining -= time_to_end` stays NaN, and with `Finite(n)` `saturating_add` eventually terminates (jumping the animation to its endpoint) while **`Infinite` never sets `exhausted`, so the `loop` at `:160` never exits**.

The other three effects, from the same one tick:

```
(fling)  after the flick scroll_top=120; ticks 159.03 / 196.19 / 231.56
         after ONE NaN tick: scroll_top=NaN; 2000 good ticks later: still NaN
         set_scroll_top(0) -> 0; one more tick -> NaN   (five repair rounds, each surviving one frame)
(longpress) 0.1s of a 0.5s threshold held, fired=0 -> after NaN, fired=1   (control: 0.2s, fired=0)
(finite anim) t=0.5 active=1 width=50 -> after -0.4 active=1 width=50 -> after NaN active=0 width=100
(frame loop) control settles quiet after 20 drained ticks; the NaN fling is quiet=false after 20001
```

`fling.rs:24` (`seconds <= 0.0`) admits NaN and `:39` (`is_done` = `velocity.abs() <= rest_speed`) makes it immortal, so `flings.retain(|_, f| !f.is_done())` (`pointer_sessions.rs:367`) keeps it forever and it returns a NaN travel on every subsequent good frame. The stall net written for exactly this case — `gestures.doc/advance_flings.md`: "a fling that went on being advanced would keep asking for frames while moving nothing" — is `(after - before).abs() < f32::EPSILON` (`ui_runtime/gestures.rs:129`), which `NaN - NaN` never satisfies. `long_press_recognizer.rs:26` has the same `seconds <= 0.0`. Recovery exists but is two calls and the runtime never performs either: a fresh touch Down removes the fling (`pointer_sessions.rs:232`), then `set_scroll_top(0)` sticks.

Not deliberate, and the sibling integrator proves it: `time_driver.rs:28` is `if delta > 0.0`, and `time_driver.doc/advance.md` says "Only a strictly positive `delta` accumulates; a negative delta (**and `NaN`**) is ignored". `Timeline::with_delay` special-cases `is_nan()`. `ui_runtime/motion.rs:79` guards its own canvas clock with `delta.max(0.0)` in the same function that hands the raw delta to the three unguarded sites at `:73`, `:74`, `:84`.

Distinct from ledger item **A9** (line 796), which is the NaN *wheel notch* into `ScrollOffsets::shift` and explicitly scopes itself out of the fling path ("the internal fling path cannot manufacture one") and asserts "an explicit `set_scroll_top(port, 0.0)` fully recovers" — false while a fling is live. Reachability: every in-tree host filters NaN by accident (`runner.rs:334`, `layer_runner.rs:195` are `if delta > 0.0`; `headless_host.rs:89` divides u64 nanos), so the door is an application driving its own clock — the documented contract (`docs/conformance-audit.md:443`), and the same reachability class the ledger already accepted for A9.

**Effort:** S. **Done means:** `advance_motion` rejects a non-finite delta at the door (or each of `Timeline::advance`, `Fling::advance`, `LongPressRecognizer::advance` flips its guard to `if delta > 0.0`); a test ticks an `infinite` timeline with `f32::NAN` and asserts it returns unchanged; a test ticks an armed 0.5 s hold with NaN and asserts it has not fired; a test flings a list, ticks NaN, and asserts the offset is finite and frame demand falls to zero.

---

### Part B — breaks an application

#### B1. Style state for a removed node is never pruned on the incremental path: 114 MB per minute of scrolling, and a frame-time collapse against the full pass it replaces

The largest item in this pass. Four lanes found it independently and two adversaries reproduced it to within 0.2 % on RSS.

**Semantic proof.** Render 20 rows, keep the 15 NodeIds about to die, render 5 rows:

```
15 nodes removed: in tree=0, computed_style() still answers for 15, layout box for 0
after 200 more frames at 5 children: 15 dead nodes still styled
after 200 churn cycles: 150 of 150 sampled dead nodes still styled
after a viewport change (full restyle): 0 still styled
```

`UiRuntime::computed_style(n)` returns `Some` for a node `tree().get(n)` returns `None` for. `styling.doc/computed_style.md` states the opposite in writing — "[`None`] when the node was never styled (**absent from the tree**, …)" — so the public API contradicts its own contract.

**The stated origin use case.** A `.list` holding a 30-row window over a 100 000-row model, 3 nodes per row, keyed by index, window sliding one row per frame — a virtualized list under a continuous scroll. `tree.len()` pinned at 122.

```
frame  500 ( 8.3s of 60Hz scrolling): rss +17588 kB   frame  8.60 ms
frame 1500 (25.0s):                   rss +49616 kB   frame 23.84 ms
frame 3000 (50.0s):                   rss +97572 kB   frame 47.01 ms
3000 frames in 76.5s wall.  frame 20 = 1.59 ms.  growth = 32.52 kB/frame = 114.3 MB per minute
```

Control, same binary, the repository's own escape hatch (`restyle_phase.rs:84`):

```
OBSCUR_FULL_RESTYLE=1
frame  500: rss +1104 kB   frame 1.446 ms
frame 3000: rss +1104 kB   frame 1.367 ms
3000 frames in 4.3s wall.  growth = 0.37 kB/frame
```

The optimisation ends up 34× slower than the path it replaces, and still climbing linearly, while leaking 97 MB.

**It does not flatten and it is not the live tree.** With a 3-row window (`tree.len()`=14, constant) out to 4000 frames — 10× the removals of the original checkpoint — the growth is the same 32 kB/frame and frame time still reaches 74 ms. A 14-node tree leaks exactly as fast as a 122-node one, so the cost scales with nodes *ever removed*. Independent runs at 8000, 10000, 16000 and 20000 frames are all dead straight. `idle` mode (the same tree re-rendered with zero churn, 400–3000 frames) is flat to the byte at 51–155 µs/frame, so it takes churn; `NO_CSS=1` leaks identically, so it is not CSS-specific; a `toggle` view whose rows return under the same keys leaks identically, so it is not keyed-identity-specific.

**The map, measured directly.** An instrumented copy outside the repo, 20 rows re-keyed per frame over a constant 22-node tree: after 100 generations `cascaded=2001 custom=2000 computed=2002`; after 1000, `20001 / 20000 / 20002` with VmRSS 23 MB → 179 MB. Under `OBSCUR_FULL_RESTYLE=1` the same 100 generations end at `21 / 20 / 22` and 5 MB. `layout_boxes().len()` stays at the live count throughout — the layout side is the correct model of what the style side should be doing.

**Attribution, two independent ways.** `valgrind --tool=massif` on the churn binary names 16 430 064 of an 18 821 689-byte peak in one chain: `hashbrown::RawTableInner::fallible_with_capacity` ← `StyleResolver::resolve` ← `StyleCoordinator::resolve_incremental` ← `RestylePhase::invalidated` ← `run_frame` ← `render_now`. `callgrind` puts the per-frame cost in `PropertyValue::uses_ch` (14.0 %) and `ComputedStyle::get` (~24 %) reached from `layout_style_from` — the conversion of stale entries, not the resolve.

**Mechanism.** `crates/application/obscur-runtime/src/orchestration/style_coordinator.rs:117-165` — `resolve_incremental` walks `Self::preorder(root, …)` over the **live** tree and only inserts (`:151` `if dirty.is_invalidated(node) || !styles.contains_key(&node)`, `:160` `styles.insert(node, computed)`); there is no removal for a key the walk no longer reaches. `restyle_phase.rs:184-241` hands the retained map straight back, `style_phase.rs:164` (`std::mem::take(self.retained)`) carries it across frames, and the field is `ui_runtime.rs:107`. A removal does **not** widen to a full pass: `restyle_phase.rs:253` `ReconciliationOp::Remove { parent, .. } => roots.push((*parent)?)` pushes the parent, so the incremental path is exactly the path a list takes. `style_resolver.rs:64` `ComputedStyle::with_capacity(COMPUTED_PROPERTIES.len())` (~124) sets the per-entry cost at ~4 kB regardless of sheet size — an empty sheet leaks 58 kB/frame, a 200-rule sheet 62 kB/frame.

**`StyleInputs.cascaded`/`.custom` leak alongside it, for a different reason than first reported.** An instrumented run counts `dirty_seen=20979 departed_skipped=0` — the `continue` at `style_matching_coordinator.rs:99-102` is taken **zero times** and cannot be taken by a departed node, because `RestyleScope::expand` returns early for a root absent from the tree (`restyle_scope.rs:36-38`, stated policy in `expand.md`) and `mark_subtree` only walks live children, so dirty ⊆ live tree always. Clearing in that branch would fix nothing. What is missing is a **removal-driven** prune: `ReconciliationOp::Remove { id, .. }` carries the removed id (`obscur-tree/src/reconciliation/reconciliation_op.rs:18,44`) and no phase uses it.

**Turn cost into a per-frame walk.** The leaked map is scanned whole every composing frame at `layout_coordinator.rs:43-59` (`layout_styles_from` maps **every** entry through `layout_style_from`, from `arrange_phase.rs:89`), `resource_bridge.rs:309`, `:392`, `:429`, `grid_areas.rs:16`, `grid_lines.rs:18`, and `layout_classifier.rs:122,:169` — so frame cost tracks the map, not the tree.

**Correction worth keeping.** "For the life of the process" is too strong. Any full restyle throws the map away and rebuilds from the live tree, and a viewport flip of 1 px every 500 frames plateaus RSS at +10.5 MB and holds frame cost at 1–5 ms across 1600 further frames of identical churn. The bound is the **interval between full restyles** — viewport change, stylesheet change, media-environment change, an unbounded scope, or the escape hatch. For a steady-state application that neither resizes nor swaps sheets — the file-manager list this toolkit exists for — that interval is unbounded in practice. That reset behaviour is also the cheapest interim mitigation, and the reason nobody has noticed.

Not deliberate. No `.doc/*.md` states retention of a dead node's style; `ui_runtime.doc/computed.md` justifies only retention *across frames*, `restyle_phase.doc/invalidated.md` states the opposite contract ("what the phase pays is proportional to the dirty set"), and every neighbour on the same frame prunes by liveness — `generated_content.rs:144` does exactly the right `retain` under a comment explaining why ("node ids carry a generation, so a dead id can never be mistaken for a live one here"), as do `motion_driver.rs:126-132`, `measure_cache.rs:53`, `arrange_cache.rs:97`, `inline_styles.rs:123`, `resource_bridge.rs:344,373`, `handler_registry.rs:111`. `image_cache.doc/new.md` calls an unbounded map "a leak that only shows itself after the program has been running".

**Effort:** M. **Done means:** `ComputedStyles`, `StyleInputs.cascaded` and `StyleInputs.custom` hold one entry per live node; `computed_style()` returns `None` for a node absent from the tree, restoring its documented contract; the `filelist` probe reports flat RSS and flat frame time over 3000 frames with no `OBSCUR_FULL_RESTYLE`; a test churns a keyed window for N frames and asserts the three map lengths equal the tree length.

#### B2. A working set larger than the image-cache budget re-loads and re-decodes every frame, forever — and the default budget is enough to trigger it

Filed against a host-set 20-byte budget; the reachable case needs no configuration at all.

```
DEFAULT budget, 600 <img> nodes (8 on screen), 60 frames:
  loader_calls=5792  per_frame=17.39ms  painted_last_frame=512  diagnostics=0
  stats=CacheStatistics { hits: 30208, misses: 5792, evictions: 5280, insertions: 5792, entries: 512 }
DEFAULT budget, same tree, 240 frames: loader_calls=21632  per_frame=15.21ms  diagnostics=0
unlimited budget, same tree, 60 frames: loader_calls=600   per_frame=6.53ms   painted=600
```

(21632 − 5792) / 180 = **88.0 loads per frame, exactly 600 − 512**, linear over 4× the iterations. Frame cost 6.53 → 17.39 ms, a 2.66× regression with a 16-byte in-memory image and a no-IO loader; with `FsResourceLoader` and real PNGs those 88 are 88 disk reads and decodes per frame. The byte cap has the same shape — a budget seating 4 of 5 visible images gives `loader_calls=64` over 60 frames and paints 4 of 5, with the missing one rotating; seating 5 gives `loader_calls=5`.

Cause: `resolve_element_images` (`resource_bridge.rs:488-502`) walks the **whole retained tree**, not the viewport, so the working set is the number of image URLs in the tree, while `DEFAULT_MAX_ENTRIES = 512` (`image_cache.rs:15`) is justified in its doc as "well above what any one screen shows". `eviction_policy.rs:19` `choose_victim` has no referenced-this-frame veto; `image_cache.rs:133-151` `insert` evicts in a loop until the newcomer fits; `resource_bridge.rs:246-250` `release_evicted` drops the victim's `image_handles` entry, which is the only guard that would have stopped the re-request next frame. A non-virtualized directory listing of 600 files, or a gallery holding five full-screen 4K photos (the byte cap "seats four"), reaches this with no configuration; a virtualized list does not.

Not deliberate: `image_cache.doc/DEFAULT_MAX_BYTES.md` names thrash as the harm being *avoided* ("a budget small enough to thrash is worse than none"), and its sizing reasoning is screen-bounded while the resolve pass is tree-bounded. The single-oversized-image case *is* deliberate and does hold (1 loader call, never retried), which is what makes this the odd one out.

Two remedies that look right and are not: the "visible this frame" veto never fires here (the churning entries are all `Offscreen` and tie), and refusing the insert would replace a reload loop with a permanently-unpainted image.

**Effort:** M. **Done means:** a tree that does not change between frames issues O(images) loader calls, not O(frames), at any budget — either a per-URL back-off keyed on "evicted while still referenced", or a viewport-bounded resolve pass. And the condition is reported: `diagnostics=0` across all 12 probe runs today, so a host has no signal that its frame time tripled.

#### B3. The surface acquirer's one-slot channel silently drops three different messages: dropping it hangs, a lost request freezes the window, and a resize burst configures the wrong size

`sync_channel::<AcquireRequest>(1)` (`surface_acquirer.rs:41`) carries `Image`, `Resize` and `Stop`, and every send is a discarded `try_send`. Reproduced against the real file copied verbatim (only the doc attributes and three GPU `use` lines rewritten) driving a stub surface whose `current_texture()` sleeps.

**Drop hangs.** A resize queued while an acquire is in flight, then the surface goes away:

```
$ timeout 30 ./acq-probe drop-loses-stop
channel holds the resize; dropping the acquirer now
exit=124        # nothing after that line ever printed
$ for t in /proc/$PID/task/*; do echo "$(cat $t/comm) wchan=$(cat $t/wchan)"; done
acq-probe          wchan=__futex_wait      # main, inside JoinHandle::join
obscur-surface-    wchan=__futex_wait      # acquire thread, parked in inbox.recv()
```

Control with nothing queued: `drop returned after 1.400203109s`. And `spawn(); request(); drop()` needs no resize at all: **40 / 40 hangs**. `Drop::drop` (`:119-131`) `try_send`s `Stop` into the full slot, then joins; the loop is `while let Ok(request) = inbox.recv()` (`:148`) and `self.requests` is a live struct field, so `recv()` can never return `Err` and the thread can never exit. Reached from `layer_runner.rs` (acquirer dropped at the end of `run`, after `resize_to_surface` at `:179`/`:233`) and `runner.rs:204` (after `acquirer.resize` at `:544`).

**A lost request freezes the window.** The windowed runner's shape, one `WindowEvent::Resized` delivered mid-acquire, three consecutive runs:

```
presented 0 frames in 5 s; pending=true; surface 800x600 (window 800x600); acquires 1
presented 0 frames in 5 s; pending=true; surface 800x600 (window 800x600); acquires 1
presented 0 frames in 5 s; pending=true; surface 800x600 (window 800x600); acquires 1
```

`resize()` fills the slot; the next redraw finds no image, calls `request()`, whose `try_send(Image)` fails and returns without scheduling anything (`:60-71`); no image is coming, so the acquire thread never rings the wake, so no redraw is ever requested. `runner.rs:300-306` and `:313-320` are the only two exits from `render_frame` that do not call `self.request_redraw()`; `about_to_wait` (`:726-741`) re-arms only on `poll_resource_tasks() > 0` or `control.has_pending()`, neither true on an idle UI. The inline comment at `:64-67` — "A full channel means a request is already in flight, which is the same state `outstanding` records" — is the invariant, and `resize` and `Drop` are what falsify it. Symptom in production: a window that stops repainting after a resize until the pointer moves.

**A resize burst keeps the oldest extent.** Three resizes while the thread is inside a 400 ms acquire: `configures seen: 1 ; surface ended at 800x600` after asking for 800x600, 900x700, 1000x800. A resize drag is exactly this shape, as is a compositor sending `Configured` + `ScaleFactorChanged` in one batch.

The layer runner escapes the second and third of these because it calls `acquirer.request()` unconditionally every turn under a 16 ms bounded wait (`layer_runner.rs:125-127`) and re-checks the extent (`:227-235`) — measured converging in 31 turns with 0 stale images dropped. It does not escape the first.

Not adversarially re-run (no verdict was issued for this lane); the evidence above is the lane's own, and the drop-hang is 40/40 deterministic.

**Effort:** M. **Done means:** dropping a `SurfaceAcquirer` with a request queued returns within one acquire; a refused `request()` is retried or reported rather than silently lost; a burst of resizes leaves the swapchain at the last extent asked for. A test drives all three against a stub surface.

#### B4. A load whose result never arrives is parked in `Loading` for the life of the runtime — and the shipped executor reaches it through a panicking loader or decoder

The send is the **last statement** of the closure at `resource_bridge.rs:566-574`, so anything that prevents it loses the load with no record. Two doors, both measured, plus a third at `:598-606` with the identical shape.

**An executor that drops the job** (`fn run_blocking(&self, _job: Box<dyn FnOnce() + Send>) {}` — the shape of `spawn_blocking` on a shut-down runtime, a bounded pool that rejects, or a worker that dies before `tx.send`), 50 frames with `poll_resource_tasks()`:

```
[dropexec]              resources=1 cached=(0,0) drawn=0 diags=[] misses:50
[baseline-no-executor]  resources=1 cached=(1,0) drawn=1 diags=[] hits:49 insertions:1
```

**The shipped executor plus a panicking decoder** — `KitBootstrap::install_blocking_executor` → `SmolSpawner::run_blocking` = `blocking::unblock(job).detach()` (`obscur-task-smol/src/spawner/smol_spawner.rs:77-79`), a perfectly well-behaved port, and an ordinary third-party-codec failure:

```
thread 'blocking-1' panicked at: third-party decoder panicked on this input
[panicking-decoder-real-executor] resources=1 cached=(0,0) drawn=0 diags=[]
```

The process survives (the pool isolates the panic) and the resource is stuck in the same silent `Loading`. **No executor misbehaviour is required.** A panicking *loader* gives the same result, against a control where the same loader returning `Err(SourceUnavailable)` emits `resource.load_failed` with the URL. A file that is simply absent reports `NotFound` on both paths, so the runtime does distinguish dead from slow for every failure it hears about — this one it never hears about.

**Stickier than first reported.** Three recoveries a host might try: stop referencing the URL for a frame and reference it again → `drawn=0 resources=1`, the handle is not released; install a working executor afterwards and render 50 more frames → `misses` climbs 50 → 151 while nothing is re-requested, because `image_handles.contains_key` at `:429`/`:505` skips the URL every frame; only `set_resource_providers` recovers it, and that is a wholesale provider reset for every URL, documented nowhere as a recovery path.

Not deliberate. `blocking_executor.md` and `load_image.md` describe the two arms as differing only in *when* the load runs ("both paths end in the same place: `apply_outcome`"). `image_handles.md` says a URL is "requested at most once" and the comment at `:429` justifies the guard as "already resolved, or already attempted (and failed)" — a job that never completes is neither. And `resolve_background_images`'s no-provider branch (`:383-410`) drives an unserviceable request straight to terminal `Failed` with a named diagnostic precisely so a caller "can tell it is dead rather than parked in `loading` behind a slow network". Partially acknowledged in `smol_spawner.doc/run_blocking.md` ("a detached task is never polled … the image is not painted"), but nothing on the runtime side records that a load can be parked forever with no diagnostic, and `resource_bridge.rs` has no in-flight registry, deadline or reaper.

**Effort:** S. **Done means:** the job is wrapped so a panic becomes a `ResourceError` on the channel, and a result that never arrives eventually reaches a terminal state with a `resource.load_failed`. A test installs a job-dropping executor and asserts the hub entry becomes `Failed` with a diagnostic; a test installs a panicking decoder on the real `SmolSpawner` and asserts the same.

#### B5. Every failure on the SVG element path is silent, while the identical raster failure beside it reports

Measured on the **real shipped stack** (`KitBootstrap::install_image_resources` + `install_vector_resources` = `FsResourceLoader` + `ImageDecoder` + `SvgVectorizer`, exactly what `App` wires at `app_parts.rs:154-155`), real files on disk, 3 frames + `poll_resource_tasks()`:

```
missing.svg (file absent)    : diags=[] resource_count=1 cached=(0,0) painted=0
missing.png (file absent)    : diags=["resource.load_failed … NotFound"]
broken.svg  (usvg refuses)   : diags=[] resource_count=1 cached=(0,0) painted=0
broken.png  (decoder refuses): diags=["resource.load_failed … DecodeFailed(\"The image format could not be determined\")"]
no providers at all, logo.svg: diags=[] resource_count=0
no providers at all, logo.png: diags=["resource.no_loader … "]
```

A typo'd icon filename is the reachable case: `.png` reports, `.svg` says nothing, in a plain `App` with everything installed. Nothing else records it either — `resource_count()` is a bare integer, DevTools' resources panel prints only that count (`devtools.rs:158-165`), and the per-URL `ResourceState` is `pub(crate) state_of` (`:110`), referenced by one unit test and unreachable from an application.

Two independent holes in `resource_bridge.rs`: `:231`, the `ResourceOutcome::Vector` `Err` arm calls `self.hub.fail(handle, version, error)` and drops the result where the `Image` arm three lines above calls `report_failure` (`:208-213`); and `:521-523`, `resolve_element_vectors` returns early when a provider is absent with no `report_unresolvable_source` — it is not even given a `ResourceReporting` (`resource_resolution.rs:84-86` passes the tree alone, unlike its three siblings), and the raster no-provider branch cannot cover it because `:475` filters vector sources out of that loop.

The docs assert the parity that is broken: `resource_bridge.doc/vector_decoder.md` — "an SVG source paints nothing — **the same policy an absent raster decoder has for a bitmap**"; `resources.doc/set_vector_decoder.md` — "exactly as an unloadable bitmap does". The raster branch's own comment at `:466-471` says why it reports: "each referenced `src` is named once on the diagnostics channel so a missing provider is loud, not silent."

**Effort:** S. **Done means:** one `report_failure` in the Vector `Err` arm, and `ResourceReporting` threaded into `resolve_element_vectors` for a `resource.no_loader` on the early return; a test asserts a missing `.svg` and a malformed `.svg` each emit exactly one diagnostic naming the URL.

#### B6. `background-image: url(*.svg)` is handed to the raster decoder and never to the installed `VectorDecoder`

Same runtime, same loader, same bytes, real CSS through `StylesheetParser` + `StylesheetCompiler`, real `ImageDecoder` + real `SvgVectorizer`:

```
REAL stack | background-image: url(icon.svg)  cached=(0,0) dl=[] 
    diags=["resource.load_failed … DecodeFailed(\"The image format could not be determined\")"]
REAL stack | <img src=icon.svg>               cached=(0,1) dl=["PushTransform","DrawPath","PopTransform"] diags=[]
REAL stack | background-image: url(pic.png)   cached=(1,0) dl=["DrawImage"] diags=[]
MOCK raster| background-image: url(icon.svg)  cached=(1,0) dl=["DrawImage"] diags=[]
```

The PNG arm rules out "backgrounds are broken generally". The mock arm is the silent-corruption variant: a permissive host raster decoder "decodes" the SVG as a bitmap and the working vector decoder is never consulted — which contradicts `install_vector_resources.md`'s own policy that an SVG must never be "rasterized behind the author's back". `resource_bridge.rs:377` `resolve_background_images` has no `is_vector_source` filter where the element path filters at `:477` and `:498`, and `resolve_element_vectors` (`:519-528`) only scans `node.attributes().image_source()`.

**The obvious one-line fix is not sufficient**, and this is the correction that matters: `scene_coordinator.rs:379` resolves a background image with `background_image_url_of(...).and_then(|url| images.get(url))` — the raster map only — while the compositor's only vector lookup is `:500`, keyed on the element `src` attribute. Filtering in the bridge would fill `vector_cache` and still emit an empty display list, and an SVG background additionally needs `background-size`/`repeat`/`position` over a vector, which the vector emit path does not express.

**Effort:** M for (a), S for (b). **Done means:** either SVG backgrounds work end to end — bridge routing plus a vector arm in `paint_background_layer`, with `background-size`/`repeat`/`position` honoured — or a diagnostic that names the real cause ("SVG is not supported as a `background-image`; use `<img src>`") instead of `DecodeFailed`. Either way the `is_vector_source` guard the element path already applies closes the silent-rasterization path.

---

### Part C — what only grows

#### C1. `ScrollOffsets.offsets` keeps one entry per node whose offset was ever written, forever

`crates/application/obscur-runtime/src/runtime/scroll_offsets.rs:24` has `set`, `shift`, `seat` and no removal of any kind — no `retain`, `remove` or `clear` anywhere in the workspace. Measured by mounting, scrolling and unmounting a scroller N times:

```
gen   250: offsets.len()=  251 dead=  250
gen  4999: offsets.len()= 5000 dead= 4999
```

`len()` equals the generation count exactly at every sample; 10× the iterations gives 10× the entries, no flattening. Consequence is memory only and modest — `FastMap<NodeId, f32>` is ~17–34 B per entry, so ~85–170 kB at 5000 — because the map is read only through `get`, `refold` builds from the live containers, and the public `scroll_top` resolves through the live tree (a dead node reads `0.0`, confirmed). Generational NodeIds make a stale entry unreachable rather than mis-attributable. Population is every node whose offset was ever *written*, a superset of "ever scrolled": `seat` also seeds from the `VirtualFor` model-replacement reset (`virtual_lists.rs:183`) and the textarea caret-follow (`ui_runtime/caret_visibility.rs:42`).

No `.doc/*.md` records a retention policy for departed nodes; the type doc's retention rationale is about relayout only, and there is no scroll-restoration feature that would need one. 18 `retain`/prune sites exist in the crate; this file is not among them.

**Effort:** S. **Done means:** one `retain` in `refold` (`:77-113`), which already holds the live tree and already runs every frame from `frame_pipeline.rs:355` — `self.offsets.retain(|node, _| fold.tree.get(*node).is_some())`, the same shape as `motion_driver.rs:126-132`. A test asserts `offsets.len()` tracks the live container count over N mount/unmount generations.

#### C2. `set_resource_providers` / `set_vector_decoder` clear `image_handles` without releasing the hub, stranding one unreachable entry per URL already loaded

`resource_bridge.rs:123` and `:146` both call `self.image_handles.clear()` with no `self.hub.release(handle)`. `image_handles` is the only place the bridge keeps a `ResourceHandle`, and `hub.release` is the only per-entry reclaim path (`dispose_namespace` is the sole alternative and nothing in the tree calls it). **One swap is enough:**

```
before swap:                  resource_count=50  cached=(50, 0)
right after ONE swap:         resource_count=50  cached=(0, 0)
after 50 more urls post-swap: resource_count=100 cached=(50, 0)
```

The pixels go; the 50 hub entries stay with no handle left to release them. The swap path breaks a bound the no-swap path respects — with a working provider pair and no swap, `retain_referenced` keeps hub and cache in step and `resource_count` is capped by `DEFAULT_MAX_ENTRIES`: `after 2000 unique urls, resource_count=512`. One swap per URL and the ceiling is gone, linear at 1×/10×/100×: `100 → 100`, `1000 → 1000`, `10000 → 10000` (rss 9392 kB, ~400 B per entry). A second shape with its own control: `hub.request` (`resource_hub.rs:38-45`) reuses an entry by `by_source` and calls `entry.retain()`, so 100 swaps of the *same* URL end at `resource_count=1` against a no-swap control's `0` — a permanently unbalanced refcount. And `set_vector_decoder` leaks the *raster* handles, because it wipes the shared "already requested" map while clearing only `vector_cache`.

Not deliberate: `set_providers.md` mentions only dropping cached images and handles; the comment at `:121` covers the abandoned `deferred_loads` and stops. Three places state release as the discipline — `image_handles.md`, `release_evicted.md`, and `:363-372` ("Released, not merely forgotten: the hub is refcounted, and an entry nothing refers to is what `release` exists to reclaim"). `retain_referenced.md` names the wholesale clear on provider replacement as the inadequate predecessor it was written to fix, so this is the one case that fix missed. The tests at `:934-1024` assert on `bookkeeping_counts()` only, never `hub.len()`, and each installs providers once.

A stock `App` leaks nothing — `app_parts.rs:151-156` installs both providers once before any frame. Reaching this needs a swap after frames have loaded, which `KitBootstrap::install_image_resources(runtime, sandbox_root)` (`kit_bootstrap.rs:232-245`) invites: it takes a sandbox root, so a host that re-roots — a file manager changing directory — swaps per navigation and strands every thumbnail loaded so far. `resource_count()` is public API and the DevTools number (`devtools.rs:160`).

**Effort:** S. **Done means:** both methods release each handle before clearing, as `release_evicted` (`:246-252`) and `retain_referenced` (`:363-372`) already do; a test asserts `hub` length returns to the referenced count after a provider swap. Second benefit: with the entries released, an in-flight load started under the old providers finds its handle gone and `hub.complete` returns `false` at `:191`, so stale pixels cannot reach the cache.

#### C3. `PseudoStates` is never pruned, so a node removed while carrying a state keeps it — and on the input-free path the map grows without bound

Render 6 rows, `set_pseudo_state(rows[5], PseudoState::HOVER)`, render, then render 2 rows:

```
tree.get(rows[5]).is_some() = false
pseudo of a removed node: PseudoState(1)
```

Growing/shrinking 6↔2 rows and setting HOVER on the last live row each frame, public API only, no input events — exactly 0.5 entries per frame, no flattening at 100×:

```
E(100 frames):   live = 2, DEAD ids still carrying a pseudo state = 50
E(1000 frames):  live = 2, DEAD ids still carrying a pseudo state = 500
E(10000 frames): live = 2, DEAD ids still carrying a pseudo state = 5000
```

Each is cloned into `StyleKey.pseudo` every frame (`style_phase.rs:202`) and walked twice by `RestyleScope::changed_states` (`restyle_scope.rs:16-31`), so it is per-frame cost as well as memory.

Two limits, both measured, and they are what keep this small. **One pointer event clears the whole map** — `recompute_pseudo_states` (`interaction_state.rs:30-40`) replaces `self.pseudo` wholesale with a projection built from the live hover chain, focus and pressed set; the same 10 000-frame churn driven through `handle_pointer` strands **0** entries. So the leak lives only where interaction state is driven programmatically and no input is delivered — the toolkit's own tests and `css_motion_proof.rs`, the only in-repo callers. **It can never be misapplied**: NodeIds are generational, the row occupying the ghost's old slot computes `PseudoState(0)` and paints blue rather than the `:hover` red, and `RestyleScope::expand` returns bounded-`true` for a dead id so a stranded entry never widens a restyle scope either.

`pseudo_states.rs:30-37` removes an entry only when it is set to empty; `pseudo_states.doc/set.md` documents only that canonicalisation and no retention policy. The precedent is one line from where the fix belongs: `drop_focus_if_removed` (`focus.rs:210-229`) runs from `frame_pipeline.rs:134` on exactly the reconciled frame that strands the entry.

**Effort:** S. **Done means:** a `retain` against the tree beside `drop_focus_if_removed`; a probe reports 0 dead ids carrying a pseudo state after 10 000 churn frames on both the `set_pseudo_state` and `handle_pointer` paths.

---

### Part D — hangs and crashes on input a host controls

#### D1. `grid-row-start` / `grid-column-start` take an unclamped i32 and layout is linear in it, so one CSS integer OOM-kills the UI process

Real CSS through `StylesheetParser` + `StylesheetCompiler`, one `.cell` in a 2×2 grid, one `render_now`, release:

```
grid-row-start: 3          ->   1.17 ms
grid-row-start: 1000       ->   1.34 ms
grid-row-start: 100000     ->  41.26 ms, rss 12 MB
grid-row-start: 2500000    ->   1.07 s,  [peak] 172.9 MB
grid-row-start: 5000000    ->   2.06 s,  [peak] 328.3 MB
grid-row-start: 10000000   ->   4.03 s,  [peak] 653.2 MB
grid-row-start: 20000000   ->   8.00 s,  [peak] 1303.4 MB
grid-row-start: 2000000000 ->   exit=124 under `timeout 45`, [peak] pinned at MemoryMax
grid-column-start: 20000000 ->  5.21 s,  [peak] 1324.5 MB
```

Dead linear on both axes: ~0.40 µs and ~68 bytes per implicit track. **Memory, not time, is the binding limit** — `grid-row-start: 2147483647` needs ~146 GB, so the frame does not finish slowly, the process dies. A five-digit typo already costs 41 ms, a dropped frame, and layout runs synchronously on the UI thread inside `render_now`. The compiler reports `unsupported: 0` at every value including `2147483647` and `99999999999` (which saturates to `i32::MAX` through `*n as i32`). Negative lines are safe — `resolve_axis` normalises from-end against `num_tracks` and clamps.

`crates/frontend/obscur-css/src/compiler/grid_placement.rs:33` — `ParsedValue::Number(n) if n.fract() == 0.0 && *n != 0.0 => Some(*n as i32)`, no bound. It reaches `grid_item_placement.rs:52-58` (`n_cols`/`n_rows_flow` grown to the anchor) and `:174-186` `mark_block`, whose `while occ.len() < row + rspan { occ.push(vec![false; width]) }` materialises one `Vec` per implicit track.

The policy exists in the same directory, twice, and was not applied here: `grid_line_span` at `:50` is `Some((*n as u32).min(u32::from(u16::MAX)) as u16)`, and `grid_track_lowering.rs:9` declares `MAX_REPEAT: usize = 1024` with the doc reason "so a pathological repetition count cannot blow up the track list". Ledger line 1223 records `repeat(1000000, 1fr)` as probed and cleared ("capped, 4 ms") — the sweep looked at the neighbouring door and never tried this one. No `.doc` in `grid_item_placement.doc/`, `grid_placement.doc/` or `grid_line.doc/` states an unbounded implicit grid as policy.

**Effort:** S. **Done means:** `grid_line_integer` clamps the way `grid_line_span` does; a test asserts `grid-row-start: 2000000000` renders in bounded time and memory, with a diagnostic naming the clamp.

#### D2. `LayerWindow::target()` is a safe method handing out `Copy + Send + Sync + 'static` raw Wayland pointers whose "must not outlive the window" contract nothing enforces

From a facade consumer with feature `shell-layer`, this compiles with no `unsafe` block and no borrow error:

```rust
fn escapes(window: LayerWindow) -> LayerWindowTarget {
    let target = window.target();
    drop(window);   // the wl_surface and the connection are destroyed here
    target          // ... and these raw pointers are still live, Send, Sync, 'static
}
```

```
$ cargo build --release --bin target_escape   # Finished
$ ./target_escape
`fn escapes(LayerWindow) -> LayerWindowTarget` COMPILED with no unsafe block
```

A safe consumer can free the objects and hand the handles to `wgpu::Instance::create_surface` — a use-after-free reachable entirely from safe code. `layer_window_target.rs:12-32` derives `Clone, Copy`, has `unsafe impl Send`/`Sync`, no lifetime parameter, and a `pub const unsafe fn new` that the safe accessor at `layer_window.rs:271-281` bypasses. The `SAFETY` comment there states an obligation ("the target's contract is that it must not outlive it") the type gives no caller a way to discharge and no way to violate visibly; `layer_window.doc/target.md` documents only a null-pointer panic. Exported to every consumer via `surface/mod.rs:9,11` and `facade/obscur-kit/src/lib.rs:95`. This is the only `unsafe` in the tree outside test fixtures.

The shipped paths are sound: in both runners the `SurfaceAcquirer` is declared after the `LayerWindow` and drops first, and its `Drop` joins the acquire thread. The hole is the public API, not the in-tree caller. Not adversarially re-run.

**Effort:** S. **Done means:** `target()` is `unsafe fn`, or `LayerWindowTarget` borrows the window (`LayerWindowTarget<'a>`), or it holds clones of the smithay objects that keep them alive. A consumer that writes the function above no longer compiles.

#### D3. A host decoder's zero-extent `Ok` is admitted to the cache as a live entry and drawn, with no diagnostic

A `ResourceDecoder` returning `Ok(DecodedResource::new(Arc::from(Vec::new()), … with_intrinsic_size(Size::new(0.0, 0.0))))`, two frames:

```
mode=zero : cached=(1,0) stats entries=1 bytes=0 diags n=0 dl=1
   DrawImage { image: SceneImage { width: 0, height: 0, pixels: Shared([]), … } }
mode=fail : cached=(0,0) entries=0 diags n=1 codes=["resource.load_failed"] dl=0
```

The byte-length guard passes because `0*0*4 == 0`. The difference that matters is the **signal**, not the pixels: an honest failure emits `resource.load_failed` at `Error` with the referring node and `RecoveryAction::Fallback` (`resource_bridge.rs:673-693`) and leaves the resource terminal; a zero-extent `Ok` sits `ready` as a 0-byte cache entry with nothing reported.

Scope is small and worth stating plainly. It is **not** reachable from the shipped decoder: the workspace enables only PNG and JPEG (`Cargo.toml:70`), both codecs reject zero dimensions themselves (`Invalid image dimensions`, `Image width or height is set to zero`), and a truncated file already yields `DecodeFailed("unexpected end of file")` — the behaviour this asks for. Nothing downstream needs teaching either: `image_tiling.rs:30` and `image_painter.rs:30` both guard `iw == 0 || ih == 0` in their first statement, and both `.doc/draw_image.md` files state it as policy. `SceneImage::new` accepting 0×0/0 bytes is pinned by the deliberate test `zero_sized_image_needs_zero_bytes` (`scene_image.rs:140-143`).

**Effort:** S. **Done means:** a `DecodeFailed("decoded image has zero extent")` beside the byte-length rejection in `resource_outcome.rs:101-106`, so a host decoder that degenerates silently is reported rather than caching a live empty entry. Defence-in-depth against a decoder already violating `resource_decoder.doc/decode.md`, not a user-reachable defect.

---

### Part E — one documentation defect and one orphan

#### E1. `set_pseudo_state`'s doc omits the ownership rule, and `:focus-within` has no producer

`styling.doc/set_pseudo_state.md` ("Records `state` as `node`'s interaction state for subsequent frames") and `docs/api/03-runtime.md:83` omit the rule stated one level down in `orchestration/pseudo_states.doc/PseudoStates.md` and `selector/pseudo_state.doc/PseudoState.md`: the map is the runtime's projection of live input, and any hover/focus/press change reseats it **wholesale** (`interaction_state.rs:30-39` assigns; `pseudo_state_projection.rs:44` starts from `PseudoStates::new()`; fired at `pointer_dispatch.rs:225` and `focus.rs:49`). Measured: a host bit set on a row is gone after one pointer move onto *any* row, and symmetrically a host `set` destroys the runtime's live HOVER bit.

That is correct behaviour with a misleading doc, not state loss — the door for host-owned `:checked`/`:disabled` is the `CHECKED`/`DISABLED` view attribute (`tree_element.rs:105-118`, ungated by local name; `style_attribute.rs:97`), which survives every pointer move and composes with `:hover` (measured: green background and red text simultaneously). `set_pseudo_state.md` also names "the input step" as its caller, and the runtime's input step does not call it. Merging instead of assigning in `recompute_pseudo_states` would be the wrong fix: it would make a stale host HOVER bit unclearable, since the projection's only way to say "no longer hovered" is the absence of an entry.

Separately: `PseudoState::FOCUS_WITHIN` is compiled (`selector_compiler.rs:182`), matched (`selector_matcher.rs:185`) and advertised in `docs/api/04-styling.md:46`, but **no code in the workspace ever sets it**, so `:focus-within` matches nothing and cannot be polyfilled through `set_pseudo_state` either.

**Effort:** S. **Done means:** the two doc sites state the ownership rule and point host-owned structural state at the view attributes; `:focus-within` either gets a producer in the focus projection or comes out of the advertised list.

---

### What held up

The negative results are the larger half of what this pass learned, and nothing else in the repository records them.

**The incremental pipeline is correct.** A differential harness compared a warm runtime driven through a mutation sequence against a fresh runtime driven straight to the same final state, on four surfaces at once: every node's complete computed style (all `PropertyId=ComputedValue` pairs plus custom properties, sorted, keyed by tree path), every node's `LayoutBox`, the entire `Scene` display-list command stream, and the hit-test answer at a grid of ~400 points.

- **1800 random mutation sequences: zero divergences.** 4 seeds × 400 trials, sequences of 2–7 states, across 3 stylesheets, 8 keyed rows shuffled and truncated, 4 text bodies, 4 viewports, 4 device scales, random hover/active/scroll. `0 of 400 random sequences diverged` on every seed. The harness is demonstrably sensitive — a control prints `same_style=false … same_scene=false` for two states differing by one class.
- **A second, independent harness: 4800 comparisons, zero divergences** (400 random sequences × 12 steps, keyed and unkeyed, display list plus sorted layout boxes, over a sheet using `:first-child`, `:last-child`, `:nth-child(2n)`, `:nth-last-child(2)`, `+`, `~` and descendant selectors).
- **43 hand-built adversarial scenarios, one divergence (A2 above).** Byte-identical warm and cold on all four surfaces for: a class toggled on/off/on/off; text short→long→short and text set to the value it already had; children added and removed in both directions; siblings reordered; keyed rows reversed, rotated, deleted from the middle; a keyed node moved between parents and back; an inherited `font-size` changed on an ancestor and changed back; a custom property changed and restored with a descendant reading it through `var()`; viewport resized and resized back; device scale 1.0↔2.0 both ways; hover in, hover out, press, release; the stylesheet replaced and replaced back; `:first-child`/`:last-child`/`:nth-child(2)`/`:nth-last-child(2)` under front-insertion and front-removal — the classic incremental-restyle break, and it holds; `+` and `~` with the `.sel` class moving between rows; the `rem` base changed through a sheet swap; an inline `style` attribute added, changed and removed; scroll-then-shrink and scroll-then-grow; percentage sizing under two resizes and a round trip.
- **Component trees converge too.** A `#[derive(Contract)]` component driven through six prop changes matches a cold render exactly (`0 divergence(s), 19 nodes`); mount/unmount 8→2→8→4 matches a cold render of the 4-component tree (`0 divergence(s), 13 nodes`); `mounted_component_count()` is correct after 1000 mount/unmount cycles.
- **Motion converges in every dimension stressed.** A `transition: width 1s linear` sampled at 0.2 s steps settles at exactly the cold value (110 px on both sides); interrupted at 50 px and reversed it settles at exactly the cold narrow value; a node removed mid-flight takes its timeline with it (`anims=1` → `0`, still 0 four seconds later); 500 settled toggles and 500 interrupted toggles both end at `anims=0 count=0`.

**Node identity is safe.** `RetainedTree` packs a full u32 generation into `NodeId` (`retained_tree.rs:17-27`), `remove` bumps the slot generation before pushing the index onto the free list (`:110-127`), and `reuse_vacated_slot` bumps it again (`:61-73`). Every side table keyed by `NodeId` is therefore protected against **misattribution** — which is precisely why C1, C3 and B1 are leaks rather than corruption. The tree arena itself genuinely recycles: `tree().len()` never moved off its live count over 3000 remount frames, and massif attributes no growth to it.

**Four of six growth workloads are flat over 5000 release frames.** Viewport resized to a fresh width every frame: 0.10 kB/frame (so the `(NodeId, constraints…)` measure-cache key does not accumulate — `MeasureCache::evict` works). Hover walking a 20-row list: 0.00. A scroll container set to a fresh offset every frame: 0.00. A unique text string every frame: 0.00. **Idle frames are genuinely idle**: 400 frames over an unchanging tree, RSS flat to the byte after warm-up, 51 µs/frame steady.

**The d2ca9005 pruning holds.** After 500 remount generations: `active_animation_count()==0`, `mounted_component_count()==0`, `resource_count()==0`. `MotionDriver::retain_live` still retains against `tree.contains` on all five collections; the image-handle and referrer retains at `resource_bridge.rs:344,373` are intact. `LayoutBoxes` is rebuilt from the live tree only — it held at 12 and 22 while the computed map passed 60 000 entries.

**Bounded caches are bounded.** `ShapeCache` is a two-generation cache whose `rotate_at` is `capacity.max(2)/2`, rounded down deliberately, with promotion on an old-generation hit so a paragraph still on screen survives rotation. `StylesheetCache` cannot grow with edits — a new revision *replaces* the entry, and an anonymous source uses a single `Option` slot. The CSS symbol interner dies with the compile that made it (`css_pipeline.rs:52`), and its `from_index` lift into the high namespace holds. The image cache's stated "too large to fit" behaviour is real: an image larger than the whole budget is loaded once, refused once, never re-loaded (1 loader call across 40 frames).

**State on a dead node is dropped where it was fixed before.** Pointer capture: `PointerSessions::live_capture_target` (`:208-219`) releases a capture whose node has left the tree, `capture_pointer` refuses a node not in the tree, and the landing path goes through it (`pointer_landing.rs:95`) — I expected a stuck-input deadlock and there is none. Focus: `drop_focus_if_removed` runs on every reconciled frame and correctly runs *after* trap reconciliation, so a closing modal restores focus one level out. Every "act on a node that does not exist" call refuses rather than inserting state: `set_scroll_top`, `scroll_by`, `scroll_into_view`, `focus_node`, `capture_pointer` on a ghost id all no-op.

**Re-entrancy into the runtime is structurally impossible, not merely discouraged.** Every application callback the frame invokes takes something other than the runtime: `CommandHandler` is `Box<dyn FnMut(&mut UiEvent)>`, the root-view builder is `FnMut() -> View`, virtual-row builders take `usize`, `CanvasPainter` takes a region, the inline-style compiler takes `&str`. "Replace the stylesheet from inside a handler", "unmount your own node from a handler" and "request a frame from inside a frame" cannot be spelled; `AppControl` queues all three and drains at the frame boundary.

**Hostile surface and clock input degrade rather than break.** `set_surface(0x0)` and back: root box 300×300 → 0×0 → 300×300, display list identical, no sticky blank frame, no stale clip. Device scale 0.0, negative, NaN and ∞ all clamp to the documented `MIN_DEVICE_SCALE = 0.1`. Two `set_surface` calls with no frame between coalesce, last write wins. A NaN physical width leaks a NaN into the viewport (`surface_geometry.rs:61-66` divides without the finiteness guard its neighbour at `:75-80` has) but the root box degrades to 0.0, no NaN reaches a child box, and the next good `set_surface` restores the baseline exactly. `advance_motion(-1.0)`, `+86400.0`, `-86400.0` are inert or correct — only NaN misbehaves (A3).

**Ports that lie are survived.** A decoder claiming 4 bytes are a 2×2 image is rejected (`"decoded image bytes do not match its intrinsic size"`). A `TextShaper` returning an empty `GlyphRun` for non-empty text, and one returning `x_advance = NaN`, both survive three consecutive frames with identical boxes and no NaN in any layout box; `begin_text_editing` + `set_editing_text` still report a consistent selection. A `DiagnosticSink` that panics inside `emit` unwinds out of `render_now` but leaves the runtime usable — the next frame returns a correct display list. Forty unresolvable `background-image` URLs produce `resources=40, cached=(0,0), pending_frames=0` and +40 kB held flat over 5000 frames: no retry storm, no wake-up loop.

**Resource lifecycle, absent a provider swap, is correct.** Version-stamped staleness holds: a superseded `complete`/`fail` is refused. 2000 frames each naming a different URL end at `resource_count()==1` with a failing loader and with no loader. An orphaned load — referenced on frame 1, dropped on frame 2 while a 400 ms load is in flight — releases its handle, applies **0** of the late result (`allocate_id` never reuses an id), and re-referencing starts a fresh load that paints on schedule. 20 runtimes dropped mid-load: all 20 loads finished with nobody to tell, no panic, no hang.

**Threads.** `LayerWaker` is unbreakable: 1 000 000 wakes single-threaded in 164 ms, 8 × 500 000 in 1.10 s, 20 000 create/drop rounds and 200 "drop the owner while a thread wakes it" rounds all ending at 6 open fds. Every `ready.lock()` in `surface_acquirer.rs` is `unwrap_or_else(|held| held.into_inner())`, so a panic on either side does not cascade. Drop order in both runners is correct — the acquirer drops before the window it borrows from. The nine crates without `#![forbid(unsafe_code)]` are deliberate: `xtask`'s `unsafe_boundary.rs:11` scopes the requirement to foundation/domain/application, and only one of the nine contains any `unsafe`.

**Panic audit.** Outside `#[cfg(test)]`, the whole domain + application + facade + components tree has 28 `unwrap`/`expect`/`panic!`/`unreachable!` sites, and every one traced is guarded by a shape the caller has already established. No reachable panic was found on a host-driven path.

---

### What was dropped

**Two findings dropped outright, both ruled deliberate policy with the reason and the remedy already in the public rustdoc.**

1. **Node-keyed listeners and command handlers are not reaped when a node unmounts** — reported independently by two lanes, and the retention is real and linear (20 000 registrations held with one node in the tree; 5000 replaced rows ≈ 5.2 kB each). But `command_routing.doc/clear_node_commands.md:4` states it verbatim, uses the word "unbounded", gives the cause and names `clear_node_commands` as the app's obligation; `event_listeners.doc/add_event_listener.md:25` and `on_command_at.md` repeat it; generational NodeIds confirm the "safe" half (a stale entry can never fire); and the supported facade route reaps automatically (`command_bindings.rs:79-84`). One lane's RSS evidence should be withdrawn regardless: a control churn with **zero** registrations grew +15 828 kB at n=2000 against the registering run's +14 344 kB, so those numbers measured B1, not the closures. Residual worth one owner decision, not a ledger item: the doc's rationale ("the runtime does not know whether the app means to re-register on the node's replacement") does not hold under generational ids, and `MotionDriver::retain_live` reaps four node-keyed maps using the opposite argument — two policies in one crate with contradictory reasons.
2. **`set_surface`/`set_viewport` schedule no frame** — the claim rested on "`set_surface.md` makes no claim either way", and that file says at `:14`: "Does not itself schedule a frame; pair with `request_frame(FrameReason::WindowChange)`." Every geometry setter in that file carries the same sentence (`set_viewport.md:3`, `set_device_scale.md:19`, `set_content_offset.md:13`), all pulled into the public rustdoc, all from one commit (e0792634). `set_environment` is documented as the deliberate exception, and `app/runner.rs:539-548` is the contract performed correctly, not a workaround. A host that follows the printed instruction gets correct geometry on the next frame.

**Three findings survive only in reduced form**, and are written above as what survived rather than as filed: the `set_pseudo_state` wipe (E1 — the premise "`:checked` is state the toolkit does not model" is false; the `CHECKED` attribute is the door and it works, so what is left is a doc defect plus the `:focus-within` orphan); the zero-extent decode (D3 — unreachable from the shipped decoder and already guarded in both renderers, so it is hardening); and the `StyleInputs` leak (folded into B1 — the growth is exactly as measured but the `continue` branch it was attributed to is taken **0 times in 20 979 dirty visits**, so the proposed fix would have changed nothing).

**Nothing was dropped as already present in the ledger**, but two items sit next to existing ones and are deliberately distinct: A3 against **A9** (line 796), which is the NaN wheel notch into `ScrollOffsets::shift` and explicitly scopes itself out of the fling path — and whose claim that "`set_scroll_top(port, 0.0)` fully recovers" A3 falsifies; and B1 against **item 9** (line 112), which is the opposite complaint (restyle scope too *wide* on a state change) and shares no mechanism.

**Four findings carry no adversarial verdict** because none was issued for that lane: the three surface-acquirer symptoms (B3) and the `LayerWindowTarget` escape (D2). Their evidence is the originating lane's own; B3's drop-hang is 40/40 deterministic and D2 is a compile-or-not fact, so both are cheap to re-check.

---

### Verdict on the incremental machinery

**Trust it to compute the right answer. Do not trust it to stay cheap.**

On correctness, this is the strongest evidence the repository has ever had for that machinery: 6600 warm-vs-cold comparisons — 1800 random sequences on four surfaces at once, 4800 more on two, 43 hand-built cases aimed at exactly the places incremental restyle classically breaks — produced **one** divergence, and it is not a style at all. `:first-child` under front-insertion, `:last-child` when the last child dies, `+`/`~` when a middle sibling disappears, keyed reorders, a node moved between parents, inherited properties, custom properties through `var()`, `rem` base changes, device-scale flips, stylesheet swaps: all converge byte-for-byte on computed style, layout boxes, the display-list command stream and the hit-test grid. `RestyleScope`'s over-approximation is doing its job, and generational `NodeId`s mean the stale state that does survive can never be mis-attributed to a live node — every leak in this pass is a leak and not a corruption, and that is a structural property, not luck.

The one divergence (A2) is a scroll offset that outlives its container, and its cousin A1 — the hover chain that is only ever written by a pointer event — are the same shape and the shape to watch: **not the style engine, but the ambient interaction state sitting beside it.** Style, layout and paint are recomputed from the tree; hover, scroll offsets and pseudo-state are carried forward and pruned by hand, or not at all. That is where a long-running runtime diverges from a fresh one today, and it is a small, enumerable surface — three maps, all with an existing prune site one line away.

On cost, the machinery is currently a pessimisation in the exact workload it was built for. A 30-row window sliding over a 100 000-row model — the file-manager list this toolkit exists for — leaks 114 MB per minute and degrades ~34× in frame time, while the full pass it replaces stays flat at 1.4 ms across the same 3000 frames and finishes in 4.3 s of wall time against 76.5 s. A 14-node tree reaches 74 ms per frame. The saving grace and the reason nobody has noticed is that a viewport change, a stylesheet change or a media-environment change throws the whole thing away and resets the clock; a resizing, theme-switching desktop app never accumulates. A tablet showing one scrolling list does.

So: the answer to "does a long-running application show the same thing a freshly-rendered one would" is **yes, on everything the style engine computes, and the evidence is now substantial**. It shows it more slowly every frame, and it disagrees on where the pointer is and where a re-shown panel is scrolled to. Fix B1 and the two prunes it implies, fix A1 and A2, and the machinery is defensible in the shape it was designed for. Until B1 is fixed, `OBSCUR_FULL_RESTYLE=1` is not a debugging flag — it is the faster path for any app that churns a list and does not resize.