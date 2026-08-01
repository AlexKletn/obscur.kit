Selects the [`FontData`] a run tagged with handle `id` draws against:
the exact match if present, otherwise the primary (id `0`) as a fallback,
otherwise `None` (the run is skipped rather than panicking).
