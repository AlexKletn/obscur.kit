Resolves `node`'s cascaded declarations, honouring the component-scope
boundary (§4.15).

A **top-level** node (not inside any component's render output) matches the
authored sheets exactly as before — the pure matcher already treats a
`::part()` selector as inert, so nothing about the ordinary path changes. A
**component-internal** node is *scoped by default*: the outside sheets' plain
selectors do not reach it, and only an exported style part
(`Component::part(name)` targeting a class the internal element carries, with
the `Component` half matching the host) crosses the boundary — plus the
system default sheet, which is not an author sheet and is never scoped (see
[`scoped_internal_declarations`]).

"Component-internal" is [`component_host`]'s answer, and a component that
declared [`StyleScope::Open`](obscur_view::component::StyleScope::Open)
(§4.15) is not a boundary to it: everything such a component renders takes
the top-level path above, which is what makes its output stylable at all.
