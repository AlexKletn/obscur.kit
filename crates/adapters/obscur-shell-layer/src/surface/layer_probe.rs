#![doc = include_str!("layer_probe.doc/_module.md")]

use obscur_geometry::Size;
use obscur_windowing::layer::LayerDisplay;
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::wlr_layer::LayerShell;
use wayland_client::Connection;
use wayland_client::globals::registry_queue_init;

use crate::error::LayerShellError;

use super::layer_state::LayerState;

#[doc = include_str!("layer_probe.doc/LayerProbe.md")]
#[derive(Debug)]
pub struct LayerProbe;

impl LayerProbe {
    #[doc = include_str!("layer_probe.doc/displays.md")]
    pub fn displays() -> Result<Vec<LayerDisplay>, LayerShellError> {
        let connection = Connection::connect_to_env()
            .map_err(|error| LayerShellError::NoDisplay(error.to_string()))?;
        let (globals, mut queue) = registry_queue_init::<LayerState>(&connection)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;
        let qh = queue.handle();

        // Asked before the outputs are read, so a caller learns "this desktop has
        // no layer surfaces" from the same answer whether they probe first or open
        // first. A list of monitors is not useful to someone who cannot place a
        // surface on any of them.
        LayerShell::bind(&globals, &qh).map_err(|_| LayerShellError::Unsupported)?;

        let mut state = LayerState::new(
            RegistryState::new(&globals),
            SeatState::new(&globals, &qh),
            OutputState::new(&globals, &qh),
            Size::new(0.0, 0.0),
        );
        // The one roundtrip that turns the registry's bare globals into outputs
        // carrying their connector names — the same roundtrip `LayerWindow::open`
        // performs, which is why the ids from here match the ids it resolves.
        queue
            .roundtrip(&mut state)
            .map_err(|error| LayerShellError::Dispatch(error.to_string()))?;

        Ok(state.displays())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probing_without_a_wayland_session_answers_rather_than_panicking() {
        // The value of this test is the shape of the failure, not the failure:
        // wherever it runs — a compositor, a CI container, an X11 session — the
        // answer is a `Result`, never a panic and never a hang.
        match LayerProbe::displays() {
            Ok(displays) => {
                // Ids are handed out from one, so none of them collides with the
                // zero a caller reaches for when it means "unset".
                for display in &displays {
                    assert!(display.id().raw() >= 1, "{display:?}");
                    assert!(!display.name().is_empty(), "{display:?}");
                }
            }
            Err(error) => {
                let text = error.to_string();
                assert!(!text.is_empty(), "a failure has to say what happened");
            }
        }
    }
}
