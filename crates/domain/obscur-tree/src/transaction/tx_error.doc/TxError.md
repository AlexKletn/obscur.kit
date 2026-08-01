A validation error produced when a [`TreeTransaction`](super::TreeTransaction)
is validated or committed.

Validation runs over the *entire* staged batch before any change is applied,
so returning one of these guarantees the live tree was left completely
untouched (atomicity). Validation itself never panics; every failure mode is
reported as one of these variants.

Marked `#[non_exhaustive]`: more validation cases may be added without a
breaking change, so callers must include a wildcard arm.
