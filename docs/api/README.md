# obscur.kit — API documentation

What the kit actually offers, written against the code rather than against the
specification.

## Two documents, not one

**`cargo doc --no-deps -p obscur-kit --features full`** is the exhaustive
per-item reference. Every public type and method in this workspace carries a
doc comment, and those comments are the authority on individual behaviour —
including the awkward parts, which they state rather than omit. Build it and
read it for "what does *this* function do".

**These pages** are what rustdoc cannot give: the shape of the whole, in the
order an application meets it, and — the part that matters most here — an honest
line between what is wired end-to-end and what merely compiles.

## Why the distinction is load-bearing

This workspace is larger than the runtime that drives it. Whole crates exist,
pass their tests, and are called by nothing in a running application. A type
existing is not evidence that anything invokes it, and a doc comment asserting
the wiring is not evidence either — a previous audit of this repo was misled by
exactly that, in both directions.

So each page marks its claims:

| Mark | Meaning |
|---|---|
| **wired** | reachable from a running app, with a production caller |
| **host-wired** | real, but only if the host installs it (the kit ships the installer) |
| **unwired** | compiles and is tested; nothing in an application path calls it |
| **partial** | works for the stated cases and stops at a named boundary |

An unmarked API is **wired**. Anything marked otherwise says what is missing.

## The pages

| | |
|---|---|
| [01 — Entry points](01-entry-points.md) | `App`, embedding a `UiRuntime`, headless |
| [02 — The view model](02-view-model.md) | `View`, `ViewNode`, props, events, virtual lists |
| [03 — The runtime](03-runtime.md) | every `UiRuntime` method, grouped by task |
| [04 — Styling](04-styling.md) | the CSS pipeline and what CSS actually does |
| [05 — Components](05-components.md) | primitives and widgets |
| [06 — Input](06-input.md) | pointer, touch, gestures, keyboard, commands, IME |
| [07 — Text and fonts](07-text-and-fonts.md) | the text stack and how a host installs it |
| [08 — Rendering](08-rendering.md) | renderers, scenes, targets, frames |
| [09 — Authoring formats](09-authoring-formats.md) | `.okui` files, the `okui!` macro, the UI IR |
| [10 — Hosting](10-hosting.md) | `KitBootstrap`, Cargo features, platform support |
| [11 — Diagnostics](11-diagnostics.md) | DevTools, the inspector, frame statistics |
| [12 — What is not wired](12-reality.md) | the verified gap list |

## Conventions

- **Signatures are copied from the source**, not paraphrased. Where a page shows
  a signature it is the real one, at the line cited.
- **Logical pixels** are CSS pixels. Physical pixels appear only where a page
  says so; the boundary is `UiRuntime::set_surface` / `set_device_scale`.
- **`InternedSymbol::named("…")`** is how a command, list or event name is
  spelled at every API that takes one.
- Line references (`file.rs:123`) were correct when written and drift as the code
  moves. The names are stable; the numbers are a convenience.

## Verifying this document

Every claim here is checkable against the workspace:

```sh
cargo run -p xtask -- check       # layer boundaries
cargo test --workspace            # 5,766 tests
cargo doc --no-deps -p obscur-kit --features full
```

Where a page says "no production caller", the check is a grep for the symbol
outside its own crate and outside tests. That is the same method the conformance
audit used, and the reason it is stated as a method rather than a conclusion:
it decays as soon as someone adds the caller.
