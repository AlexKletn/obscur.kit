#![doc = include_str!("theme_catalog.doc/_module.md")]

use super::dark_saas_theme::DARK_SAAS_CSS;
use super::flat_theme::FLAT_CSS;
use super::fluent_theme::FLUENT_CSS;
use super::lara_theme::LARA_CSS;
use super::material_theme::MATERIAL_CSS;
use super::metro_theme::METRO_CSS;
use super::minimal_theme::MINIMAL_CSS;
use super::skeuomorphic_theme::SKEUOMORPHIC_CSS;

#[doc = include_str!("theme_catalog.doc/MATERIAL_INDEX.md")]
pub const MATERIAL_INDEX: usize = 1;

#[doc = include_str!("theme_catalog.doc/THEME_NAMES.md")]
pub const THEME_NAMES: [&str; 8] = [
    "Minimal",
    "Material",
    "Dark SaaS",
    "Flat",
    "Skeuomorphic",
    "Metro",
    "Lara",
    "Fluent",
];

#[doc = include_str!("theme_catalog.doc/themes.md")]
#[must_use]
pub fn themes() -> &'static [(&'static str, &'static str)] {
    &[
        ("Minimal", MINIMAL_CSS),
        ("Material", MATERIAL_CSS),
        ("Dark SaaS", DARK_SAAS_CSS),
        ("Flat", FLAT_CSS),
        ("Skeuomorphic", SKEUOMORPHIC_CSS),
        ("Metro", METRO_CSS),
        ("Lara", LARA_CSS),
        ("Fluent", FLUENT_CSS),
    ]
}
