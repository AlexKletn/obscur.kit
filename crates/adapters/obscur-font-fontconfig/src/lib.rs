//! System font enumeration on Linux via fontconfig (spec §5.18; §2.7
//! target-specific providers).
//!
//! A self-shaping toolkit needs font **bytes**, so "use the system font" only
//! becomes meaningful once something has asked the host *which file* a family
//! lives in. This crate answers that by invoking the conventional fontconfig CLI
//! (`fc-match`, `fc-list`) through [`std::process::Command`] — dependency-free and
//! synchronous — the same shape `obscur-system-linux`'s `xdg-open` provider uses.
//! Linking `libfontconfig` (via the `fontconfig`/`font-kit` crates) is a future
//! refinement; it buys a real API but costs a C toolchain dependency that nothing
//! else in this workspace needs.
//!
//! The crate reports *where faces are*, never their bytes: reading files is the
//! facade's business (§2.7 keeps host reach explicit), and it is the facade that
//! turns these into a font set.
//!
//! # Two tools, because they answer different questions
//!
//! `fc-match` **always answers**. Asked for a family that is not installed it
//! returns the system default with exit status 0 — that is substitution, which is
//! exactly right for a CSS *generic* (`sans-serif` is a request for whatever the
//! host nominates) and exactly wrong for a *named* family (`font-family: Inter`
//! must fall through to the next candidate when Inter is absent, not silently
//! become Noto Sans). So [`FontconfigSource::match_generic`] shells out to
//! `fc-match` and [`FontconfigSource::find_family`] to `fc-list`, which returns
//! nothing for an absent family and can therefore answer `None` honestly.

#![forbid(unsafe_code)]

pub mod source;
