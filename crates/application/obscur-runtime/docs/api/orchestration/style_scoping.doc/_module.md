Scope boundaries and `::part()` matching for authored CSS against the
retained tree (§2.12 exported style parts, §4.15 scoped-by-default).

A component's rendered output is *scoped by default*: an outside stylesheet's
ordinary selectors do not reach a component-internal element. The one door
through the boundary is the exported style part — `Component::part(name)` —
which matches an internal element the component exports under `name` while
the rest of the compound matches the component **host**.

§4.15 has a second half, and it lives here too: "*explicitly open/global
scopes are possible*". A component whose contract declares
[`StyleScope::Open`](obscur_view::component::StyleScope::Open) is mounted with
[`NodeFlags::OPEN_STYLE_SCOPE`] and closes no boundary at all, so everything
it renders is matched exactly as authored content is. Until that existed a
component's output could be styled by nothing whatsoever — there is no
producer for the `ComponentModule` cascade origin yet, so `::part()` was the
only route to any declaration reaching an internal element, and the external
consumer audit's finding 2-1 recorded the consequence: "a component-rendered
subtree is matched by no selector".

The pure `obscur-style` matcher cannot decide this: it has no view of the
component-host boundary, which lives on the retained tree
([`NodeFlags::COMPONENT_HOST`]). These helpers supply that view so the
runtime's style-matching coordinator (and the generated-content resolver) can
anchor a `::part()` rule on the host while gating the crossing on the export.
