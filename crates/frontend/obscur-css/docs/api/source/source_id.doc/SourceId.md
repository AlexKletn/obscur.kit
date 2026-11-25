A stable identifier for a CSS source (spec §2.5).

Each parsed stylesheet is assigned a `SourceId` so that spans and diagnostics
can refer back to their originating source without carrying the source text.
The identifier is an opaque `u32` newtype; callers mint ids through
[`SourceId::from_raw`] and recover the underlying value with
[`SourceId::raw`].

The reserved value [`SourceId::ANONYMOUS`] denotes a source with no stable
identity, such as an ad-hoc stylesheet parsed from an in-memory string.
