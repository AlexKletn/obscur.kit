A node's box geometry must be recomputed without its style having changed.
Maps to [`InvalidationKind::Layout`].

The cause a caller states when it knows the *result* of a style rather than
the style itself — it asks for a relayout and nothing more, so the style
phase is not dirtied. [`UiOperation::InvalidateLayout`] is exactly that
statement, and this is what it means.

[`UiOperation::InvalidateLayout`]: crate::transaction::UiOperation::InvalidateLayout
