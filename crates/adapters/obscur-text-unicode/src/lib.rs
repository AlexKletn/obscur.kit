//! Real Unicode itemization adapter over [`unicode-script`](https://docs.rs/unicode-script)
//! and [`unicode-bidi`](https://docs.rs/unicode-bidi) (spec §5.10).
//!
//! This crate implements the domain's
//! [`TextItemizer`](obscur_text::shaping::TextItemizer) port with real Unicode
//! tables: [`UnicodeTextItemizer`] splits text into uniform runs first by the
//! resolved Unicode Bidirectional Algorithm embedding **level** (via
//! `unicode-bidi`) and then, within each directional run, by the per-character
//! **script** property (via `unicode-script`). The items it returns tile the
//! whole input in logical order — contiguous, non-overlapping byte ranges that
//! cover `0..text.len()` with no gaps — so a downstream
//! [`TextShaper`](obscur_text::shaping::TextShaper) only ever sees a
//! single-script, single-direction run.
//!
//! Script-neutral characters — those whose script is `Common` (spaces,
//! punctuation, digits) or `Inherited` (combining marks) — never start a new
//! run: they attach to the surrounding run, and a leading neutral prefix adopts
//! the script of the run that follows it (falling back to
//! [`ScriptTag::COMMON`](obscur_text::shaping::ScriptTag::COMMON) when the whole
//! text is neutral). Bidi level resolution is seeded by the caller's base
//! direction; each emitted item's direction follows its level parity (even =
//! left-to-right, odd = right-to-left).
//!
//! The external Unicode crates are confined to this adapter; the `obscur-text`
//! domain stays table-free and consults these ports through the trait. Font
//! coverage and per-script shaping features are out of scope here — this stage
//! only classifies text into runs.
#![deny(unsafe_op_in_unsafe_fn)]

mod itemizer;
mod script_mapping;

pub use itemizer::UnicodeTextItemizer;
