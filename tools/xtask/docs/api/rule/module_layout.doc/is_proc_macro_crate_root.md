Whether `file` is the `lib.rs` of a `proc-macro = true` crate.

Rust requires procedural macro entry points to live in the crate root. These
attributed functions are allowed there. The expansion logic lives in modules.
