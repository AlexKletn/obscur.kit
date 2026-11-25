use super::popup_content::PopupContent;
use crate::identity::NodeId;
use crate::shell_layer::event::PopupActivation;
use crate::windowing::layer::{LayerEdge, LayerPopupFrame};

pub(super) enum PopupRequest {
    Show(NodeId, LayerEdge, LayerPopupFrame),
    View(
        NodeId,
        LayerEdge,
        (u32, u32),
        PopupContent,
        Option<PopupActivation>,
    ),
    Close,
}
