Satisfies `request` and returns the honest outcome (spec §6.17).

A provider that cannot capture the request's source returns
[`CaptureResult::Unsupported`] rather than panicking.
