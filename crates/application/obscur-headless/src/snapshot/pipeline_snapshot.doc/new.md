Builds a snapshot directly from already-captured stage data.

The rendered image is left absent; attach one with
[`with_image`](Self::with_image). Prefer [`capture`](Self::capture),
which derives every backend-independent stage from a retained tree in one
call; this constructor exists for callers that already hold the
individual stage artefacts.
