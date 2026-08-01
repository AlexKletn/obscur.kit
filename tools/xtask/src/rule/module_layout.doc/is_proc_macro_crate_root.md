Whether `file` is the `lib.rs` of a `proc-macro = true` crate.

Such a root is exempt: Rust *requires* `#[proc_macro]` / `#[proc_macro_derive]`
entry points to live in the crate root, so the file cannot be pure structure
however the crate is organised — the rule would forbid what the language
mandates. The convention's intent still holds by construction: the entry
points stay thin adapters and the expansion logic lives in modules.
