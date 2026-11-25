//! Everything a retained style map was produced from: [`StyleKey`].

use obscur_geometry::Size;
use obscur_style::matching::Stylesheet;
use obscur_style::property::EnvironmentValues;

use crate::orchestration::PseudoStates;

#[doc = include_str!("../../docs/api/runtime/style_key.doc/StyleKey.md")]
#[derive(Debug, Clone)]
pub(crate) struct StyleKey {
    /// The viewport the media environment and every `vw`/`vh` were resolved
    /// against.
    pub(crate) viewport: Size,
    /// The host-reported environment values every `env()` was resolved against
    /// (spec §6.15). Here for exactly the reason `viewport` is: a key that did
    /// not name it would call the retained styles reusable after a keyboard
    /// appeared, and the padding that was supposed to move out of its way would
    /// silently stay where it was.
    pub(crate) environment: EnvironmentValues,
    /// The sheets that were matched, UA default first — the exact slice
    /// `build_inputs_with_pseudo_env` saw.
    pub(crate) sheets: Vec<Stylesheet>,
    /// The interaction state the retained inputs/styles were matched under. The
    /// baseline this frame's map is diffed against to find the restyle roots.
    pub(crate) pseudo: PseudoStates,
}
