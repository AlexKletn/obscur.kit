The accumulated outcome of a validation pass over a UI IR document
(spec §2.5).

A result collects two buckets of [`UiIrValidationError`]: fatal `errors` and
non-fatal `warnings`. A freshly [`new`](UiIrValidationResult::new) (or
[`default`](Default::default)) result is empty, and an empty error bucket
means [`is_valid`](UiIrValidationResult::is_valid) — warnings never affect
validity. Results compose via [`merge`](UiIrValidationResult::merge), letting
a walk fold child outcomes into a parent's.
