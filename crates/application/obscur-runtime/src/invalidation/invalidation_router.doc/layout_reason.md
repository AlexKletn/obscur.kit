Maps a runtime [`InvalidationReason`] to the [`LayoutInvalidationReason`] the
layout domain should record for the affected node.

Every cause maps to a layout reason because each ultimately feeds layout:

| [`InvalidationReason`]  | [`LayoutInvalidationReason`] |
|-------------------------|------------------------------|
| `StyleChanged` / `InlineStyleChanged` / `PseudoStateChanged` / `CustomPropertyChanged` | `StyleChanged` |
| `LayoutChanged`         | `Requested`                 |
| `TreeChanged`           | `TreeChanged`               |
| `ContentChanged`        | `ContentChanged`            |
| `EnvironmentChanged`    | `EnvironmentChanged`        |
