use super::layer_control::LayerControl;
use super::popup_request::PopupRequest;
use crate::geometry::Size;
use crate::shell_layer::event::PopupActivation;
use crate::windowing::layer::{LayerPopupEvent, LayerSettings, LayerSurfaceState};

/// The shared cell both sides of the handle read and write.
#[derive(Default)]
pub(super) struct LayerControlState {
    pub(super) pending: Option<LayerSettings>,
    pub(super) popup: Option<PopupRequest>,
    pub(super) popup_handler: Option<PopupHandler>,
    pub(super) popup_frame_requested: bool,
    pub(super) popup_activation: Option<PopupActivation>,
    pub(super) close_requested: bool,
    pub(super) size: Size,
    pub(super) scale: f32,
    pub(super) state: LayerSurfaceState,
}

type PopupHandler = Box<dyn FnMut(&LayerControl, LayerPopupEvent)>;
