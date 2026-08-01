Attaches (or replaces) the primary host font (run font handle `0`).

Back-compat shim over [`Self::register_font`]: single-font hosts keep
working, and every untagged run renders against this face. Until this (or
[`Self::register_font`]) is called, text runs are skipped.
