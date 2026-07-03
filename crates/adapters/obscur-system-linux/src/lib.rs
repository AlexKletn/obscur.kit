//! Linux system-service provider adapters (spec §6.9 dialogs, §6.11 shell, §6.12
//! notifications; §2.7 target-specific providers).
//!
//! This crate adapts the `obscur_system` dialog, shell, and notification ports to
//! Linux by invoking the conventional freedesktop CLI tools (`zenity`,
//! `xdg-open`, `notify-send`) through [`std::process::Command`] — dependency-free
//! and synchronous. Portal-based implementations (`xdg-desktop-portal` via
//! `ashpd`) are a future refinement.

#![forbid(unsafe_code)]

pub mod provider;
