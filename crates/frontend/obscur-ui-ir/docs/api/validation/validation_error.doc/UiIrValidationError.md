A single structural fault discovered while validating a [`UiIrDocument`]
(spec §2.5).

Each variant names one well-formedness rule the IR tree violated. Faults
carry enough context to point a diagnostic back at the offending node — most
variants embed the stable node id, exposed uniformly via
[`node_id`](UiIrValidationError::node_id). The same type is used for both
hard errors and non-fatal warnings; whether a given fault is fatal is decided
by which bucket it lands in on the [`UiIrValidationResult`], not by the
variant itself.

[`UiIrDocument`]: crate::document::UiIrDocument
[`UiIrValidationResult`]: crate::validation::UiIrValidationResult
