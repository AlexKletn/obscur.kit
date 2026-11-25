This build has no way to open an ordinary window.

A build configured with a desktop-surface backend and no windowing one, running
an `App` that never called [`App::layer`](crate::App::layer). A configuration
mistake rather than a platform limit, which is why the message names the fix.
