Maps a runtime [`InvalidationReason`] to the [`StyleInvalidationReason`] the
style domain should record for the affected node.

Only the style-affecting causes map to a concrete style reason:

| [`InvalidationReason`]              | [`StyleInvalidationReason`]        |
|-------------------------------------|------------------------------------|
| `StyleChanged`                      | `DeclarationsChanged`              |
| `InlineStyleChanged`                | `InlineStyleChanged`              |
| `PseudoStateChanged`                | `PseudoStateChanged`              |
| `CustomPropertyChanged`             | `CustomPropertyChanged`           |
| `TreeChanged` / `ContentChanged` / `EnvironmentChanged` | `None`         |

A `None` result means the change dirties the style phase (when its kind
[`needs_restyle`](crate::invalidation::InvalidationKind::needs_restyle)) but
does not correspond to a single-node style edit, so no per-node reason is
recorded for it.
