Attaches (or replaces) the primary host font (run font handle `0`).

Back-compat shim over [`Self::register_font`]: single-font hosts keep
working, and every untagged run renders against this face. Until this (or
[`Self::register_font`]) is called, text runs are skipped. Like
[`Self::register_font`] it shares the caller's bytes rather than copying them.
