The probe: the facade's **real** `FsResourceLoader`, wrapped so the test can
read back which thread `load` was called on.

Wrapping rather than mocking keeps the loaded bytes, the decode and the paint
real; the only thing added is the `ThreadId` record. This is the test's
instrument — the executor, the runtime path and the decoder are all
production.
