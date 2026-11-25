Minimal constraint / overlay layout (spec §4.1, §4.2).

Every child is constrained to the container's content box and placed at the
content origin, so all children **overlay** one another at the top-left. This
is the simplest possible constraint context: children share the container's
bounds rather than being related to one another.

# Algorithm

1. Each child is measured under the container's content area (its
   `constraints.max` minus `padding`, clamped to be non-negative), loosened
   so a child may be smaller than the container.
2. Each child is placed at `(padding.left, padding.top)`.
3. The node size is the container's own `width` / `height` when definite
   (resolved against `constraints.max`), otherwise the largest child size
   plus the container `padding`. The result is clamped into `constraints`.

# Minimal constraint context

This is deliberately minimal: there are **no relative inter-child
constraints** (align/anchor edges to sibling edges, chains, bias, guidelines,
barriers). Those belong to a fuller constraint solver and are future work;
here every child is simply pinned to the container origin and overlaid.
