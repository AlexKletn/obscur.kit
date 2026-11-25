use crate::runtime::UiRuntime;
use crate::view::view::View;

pub(super) struct PopupContent {
    pub(super) build: Box<dyn FnMut() -> View>,
    pub(super) configure: Box<dyn FnOnce(&mut UiRuntime)>,
}
