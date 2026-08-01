§3.4 — "The synchronous frame phase must stay short" (with §5.15 "async
load/decode/compile", §5.21 "placeholders and fallback first").

A custom resource loader must never be invoked while the synchronous frame
phase is in progress.
