A reference [`ResourceLoader`] that resolves only the bytes-in-hand sources
(spec §5.15).

`InlineLoader` serves the [`ResourceSource::Embedded`] and
[`ResourceSource::Memory`] kinds, whose bytes are already present, by copying
them into an [`Arc<[u8]>`]. It cannot resolve file, URL, document, or
generated sources — those require a host loader — so for them it honestly
reports [`ResourceLoader::can_load`] `== false` and returns
[`ResourceError::SourceUnavailable`] from [`ResourceLoader::load`]. Its
refusals are correct behavior, not stubs waiting to be filled in.
