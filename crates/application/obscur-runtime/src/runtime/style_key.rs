#![doc = include_str!("style_key.doc/_module.md")]

use obscur_geometry::Size;
use obscur_style::matching::Stylesheet;
use obscur_style::property::EnvironmentValues;

use crate::orchestration::PseudoStates;

#[doc = include_str!("style_key.doc/StyleKey.md")]
#[derive(Debug, Clone)]
pub(crate) struct StyleKey {
    #[doc = include_str!("style_key.doc/viewport.md")]
    pub(crate) viewport: Size,
    #[doc = include_str!("style_key.doc/environment.md")]
    pub(crate) environment: EnvironmentValues,
    #[doc = include_str!("style_key.doc/sheets.md")]
    pub(crate) sheets: Vec<Stylesheet>,
    #[doc = include_str!("style_key.doc/pseudo.md")]
    pub(crate) pseudo: PseudoStates,
}
