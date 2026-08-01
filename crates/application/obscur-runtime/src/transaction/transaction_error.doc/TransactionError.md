A typed error produced when a [`UiTransaction`] fails validation.

Per spec §3.2, normal UI commits are *validated before application*: a batch
that is not well-formed is rejected and nothing is applied. These errors are
returned (never panicked) so the caller can decide how to recover.

This enum is `#[non_exhaustive]`: later phases may add validation rules and
corresponding variants without a breaking change.

[`UiTransaction`]: crate::transaction::UiTransaction
