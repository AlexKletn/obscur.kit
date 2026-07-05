//! Real Knuth-Liang hyphenation adapter over the
//! [`hyphenation`](https://docs.rs/hyphenation) crate (spec §5.10).
//!
//! This crate implements the domain's
//! [`Hyphenator`](obscur_text::layout::Hyphenator) port with real language
//! patterns: [`EnglishHyphenator`] loads the embedded US-English Knuth-Liang
//! dictionary and returns the interior byte offsets at which a word may be
//! hyphenated, respecting the dictionary's minimum prefix/suffix bounds.
//!
//! The pattern tables live only here, behind the xtask boundary; the pure
//! `obscur-text` domain stays table-free and consults this port through the
//! trait object the runtime installs via
//! [`set_hyphenator`](obscur_text::layout::Hyphenator).
#![deny(unsafe_op_in_unsafe_fn)]

mod english_hyphenator;

pub use english_hyphenator::EnglishHyphenator;
