The custom / extension-point layout (spec §4.1, §4.2).

A real custom formatting context is user code: any type implementing
[`LayoutAlgorithm`] *is* a custom layout. `CustomLayout` is not that
mechanism — it is a **labeled fallback**: a named passthrough that lays its
children out in a single vertical stack, serving as a sane default and a
reference implementation for the extension point.

# Name

The `name` is a diagnostic identifier for the custom context (e.g. shown in
layout traces); it does not affect the geometry.

# Algorithm (vertical-stack default)

Behaves like a minimal flow: each child is measured under the content width
(`constraints.max.width` minus horizontal `padding`) with its natural height,
stacked top-to-bottom starting at the content origin, and separated from the
previous child by `style.gap`. The node size is the content width plus
padding by the accumulated child height plus padding, clamped into
`constraints`.

# Production custom layouts

Production code implements [`LayoutAlgorithm`] directly for its own type
rather than configuring this one; `CustomLayout` exists so the extension
point has a concrete, labeled default with defined behavior.
