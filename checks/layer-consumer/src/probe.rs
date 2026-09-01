#![doc = include_str!("probe.doc/_module.md")]

use obscur_kit::windowing::display::DisplayId;
use obscur_kit::windowing::layer::LayerDisplay;
use obscur_kit::{App, AppError};

#[doc = include_str!("probe.doc/DisplayChoice.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayChoice {
    #[doc = include_str!("probe.doc/Chosen.md")]
    Chosen(DisplayId),
    #[doc = include_str!("probe.doc/NoSuchDisplay.md")]
    NoSuchDisplay(Vec<String>),
    #[doc = include_str!("probe.doc/NoDesktop.md")]
    NoDesktop,
}

#[doc = include_str!("probe.doc/choose_display.md")]
pub fn choose_display(wanted: &str) -> Result<DisplayChoice, AppError> {
    let displays = match App::displays() {
        Ok(displays) => displays,
        // The one failure an application is expected to handle rather than
        // propagate: this desktop has no layer surfaces, so there is nothing to
        // choose between and the caller should open a window instead.
        Err(AppError::LayerUnsupported) => return Ok(DisplayChoice::NoDesktop),
        Err(other) => return Err(other),
    };
    Ok(displays
        .iter()
        .find(|display| display.name() == wanted)
        .map_or_else(
            || {
                DisplayChoice::NoSuchDisplay(
                    displays.iter().map(|d| d.name().to_owned()).collect(),
                )
            },
            |display| DisplayChoice::Chosen(display.id()),
        ))
}

#[doc = include_str!("probe.doc/first_display.md")]
#[must_use]
pub fn first_display(displays: &[LayerDisplay]) -> Option<DisplayId> {
    displays.first().map(LayerDisplay::id)
}
