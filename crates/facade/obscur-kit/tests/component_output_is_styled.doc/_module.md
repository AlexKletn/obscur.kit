Both halves of §4.15, on real frames: a component's rendered output is
**scoped by default**, an "explicitly open" component's output is styled by
the document exactly as authored content is, and the system default (UA)
sheet reaches inside either.

# What this file was written from

The external consumer audit's second-pass finding 2-1 — "a component-rendered
subtree is matched by no selector". Its control experiment is
[`the_documents_selectors_stop_at_a_scoped_components_boundary`] below, and it
reproduced exactly as reported: one screen, one sheet, one frame, two nodes
identical apart from provenance, and only the authored one is styled.

It is not, however, matching that is broken. The mechanism is deliberate:
`orchestration::style_scoping::component_host` finds the nearest
`NodeFlags::COMPONENT_HOST` ancestor and the style-matching coordinator hands
any node that has one to `scoped_internal_declarations`, where only an
exported `::part()` crosses. That is §4.15's "component stylesheets are
scoped by default", and `spec_ch2_contracts.rs` pins it. (The audit's grep
looked for it under `crates/domain/obscur-style/src/matching/` and came up
empty because the boundary is a *retained-tree* fact, which the pure matcher
deliberately cannot see.)

What was genuinely missing is everything the closing half implies but nothing
provided:

* §4.15's other bullet — "**explicitly open/global scopes are possible**" —
  had no implementation, so no component could opt out;
* there is no producer for the `ComponentModule` cascade origin, so a
  component cannot ship a scoped sheet of its own either;
* the system default sheet was scoped out with the author sheets, so a
  `<button>` a component rendered was not a button.

Together those made the tier unusable for anything visual, which is the
audit's verdict and it stands. `StyleScope::Open` and the UA crossing are
what close it; the default is untouched.

# Why a computed style and not rendered text

`okui_user_component.rs` and its siblings assert on rendered **text**, which
is why this survived: text proves a component mounted and that its props
decoded, and says nothing about whether the nodes it produced are visible to
the cascade. Every assertion here reads a computed value off the runtime.
