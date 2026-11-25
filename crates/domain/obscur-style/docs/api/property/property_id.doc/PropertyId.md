A typed identifier for a known longhand CSS property.

Only longhands are represented; shorthands expand to longhands before the
cascade sees them. This is `Copy + Eq + Hash` so it can key maps in the
cascade and computed-style storage cheaply.
