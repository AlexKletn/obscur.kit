//! Asking the compositor what displays exist, without opening a surface.

use obscur_capability::SupportLevel;
use obscur_geometry::Size;
use obscur_windowing::layer::LayerDisplay;
use obscur_windowing::port::{LayerCapabilities, LayerError, LayerFeature, LayerPlatform};
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::RegistryState;
use smithay_client_toolkit::seat::SeatState;
use smithay_client_toolkit::shell::wlr_layer::LayerShell;
use wayland_client::Connection;
use wayland_client::globals::registry_queue_init;

use crate::error::LayerShellError;

use super::layer_state::LayerState;

#[doc = include_str!("../../docs/api/surface/layer_probe.doc/LayerProbe.md")]
#[derive(Debug)]
pub struct LayerProbe;

impl LayerProbe {
    #[doc = include_str!("../../docs/api/surface/layer_probe.doc/displays.md")]
    pub fn displays() -> Result<Vec<LayerDisplay>, LayerShellError> {
        let connection = Connection::connect_to_env().map_err(LayerShellError::NoDisplay)?;
        let (globals, mut queue) =
            registry_queue_init::<LayerState>(&connection).map_err(LayerShellError::Registry)?;
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
            None,
        );
        // The one roundtrip that turns the registry's bare globals into outputs
        // carrying their connector names — the same roundtrip `LayerWindow::open`
        // performs, which is why the ids from here match the ids it resolves.
        queue
            .roundtrip(&mut state)
            .map_err(LayerShellError::Roundtrip)?;

        Ok(state.displays())
    }
}

impl LayerPlatform for LayerProbe {
    fn capabilities(&self) -> Result<LayerCapabilities, LayerError> {
        let connection = Connection::connect_to_env().map_err(|error| match error {
            wayland_client::ConnectError::NoCompositor
            | wayland_client::ConnectError::NoWaylandLib => LayerError::Unsupported,
            error => LayerError::backend(error),
        })?;
        let (globals, queue) =
            registry_queue_init::<LayerState>(&connection).map_err(LayerError::backend)?;
        LayerShell::bind(&globals, &queue.handle()).map_err(|_| LayerError::Unsupported)?;
        let fractional = globals.contents().with_list(|list| {
            ["wp_fractional_scale_manager_v1", "wp_viewporter"]
                .iter()
                .all(|interface| list.iter().any(|global| global.interface == *interface))
        });
        let version = globals.contents().with_list(|list| {
            list.iter()
                .find(|global| global.interface == "zwlr_layer_shell_v1")
                .map_or(0, |global| global.version)
        });
        let popup = globals.contents().with_list(|list| {
            ["xdg_wm_base", "wl_shm"]
                .iter()
                .all(|interface| list.iter().any(|global| global.interface == *interface))
        });
        let text_input = globals.contents().with_list(|list| {
            list.iter()
                .any(|global| global.interface == "zwp_text_input_manager_v3")
        });
        Ok(LayerCapabilities::from_support(|feature| {
            layer_support(feature, fractional, version, popup, text_input)
        }))
    }
}

pub(super) fn layer_support(
    feature: LayerFeature,
    fractional: bool,
    version: u32,
    popup: bool,
    text_input: bool,
) -> SupportLevel {
    match feature {
        LayerFeature::Popup if !popup => SupportLevel::Unavailable,
        LayerFeature::TextInput if !text_input => SupportLevel::Unavailable,
        LayerFeature::OnDemandKeyboard if version < 4 => SupportLevel::Unavailable,
        LayerFeature::FractionalScale if !fractional => SupportLevel::Emulated,
        _ => SupportLevel::Native,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_and_live_adapter_use_the_same_feature_semantics() {
        assert_eq!(
            layer_support(LayerFeature::Bands, false, 4, false, false),
            SupportLevel::Native
        );
        assert_eq!(
            layer_support(LayerFeature::FractionalScale, false, 4, false, false),
            SupportLevel::Emulated
        );
        assert_eq!(
            layer_support(LayerFeature::FractionalScale, true, 4, false, false),
            SupportLevel::Native
        );
        assert_eq!(
            layer_support(LayerFeature::Popup, true, 4, false, false),
            SupportLevel::Unavailable
        );
        assert_eq!(
            layer_support(LayerFeature::Popup, true, 4, true, false),
            SupportLevel::Native
        );
        assert_eq!(
            layer_support(LayerFeature::ExclusiveKeyboard, true, 1, false, false),
            SupportLevel::Native
        );
        assert_eq!(
            layer_support(LayerFeature::OnDemandKeyboard, true, 3, false, false),
            SupportLevel::Unavailable
        );
        assert_eq!(
            layer_support(LayerFeature::OnDemandKeyboard, true, 4, false, false),
            SupportLevel::Native
        );
    }

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
