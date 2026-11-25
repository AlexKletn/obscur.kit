The severity of a [`TextDiagnostic`](super::TextDiagnostic), ordered least-to-most severe
(spec §5.13).

The [`PartialOrd`]/[`Ord`] derivation follows declaration order, so
[`Info`](Self::Info) `<` [`Warning`](Self::Warning) `<` [`Error`](Self::Error);
a consumer can filter by a minimum severity threshold.
