The ordered pattern-to-target mapping a router resolves against (spec §6.13
route-to-screen/component mapping, deep-link mapping).

Each registered entry pairs a [`RoutePattern`] with an application-chosen
**target id** — a screen or component name the toolkit stores verbatim and
never interprets. Resolution walks the registrations in order and returns the
first pattern that matches, so **registration order is priority**: register
more specific patterns before broader (for example wildcard) ones.
