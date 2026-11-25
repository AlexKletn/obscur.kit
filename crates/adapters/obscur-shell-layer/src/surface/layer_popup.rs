//! Native xdg-popup presentation for a layer-shell parent.

use obscur_geometry::{Point, Rect, Size};
use obscur_interaction::pointer::{
    PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId, WheelEvent,
};
use obscur_windowing::layer::{LayerEdge, LayerPopupEvent, LayerPopupFrame};
use obscur_windowing::port::LayerError;
use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::seat::pointer::{
    PointerEvent as NativePointer, PointerEventKind as NativeKind,
};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shell::wlr_layer::LayerSurface;
use smithay_client_toolkit::shell::xdg::popup::{Popup, PopupConfigure, PopupHandler};
use smithay_client_toolkit::shell::xdg::{XdgPositioner, XdgShell};
use smithay_client_toolkit::shm::{
    Shm, ShmHandler,
    slot::{Buffer, SlotPool},
};
use wayland_client::protocol::{wl_shm, wl_surface::WlSurface};
use wayland_client::{Connection, QueueHandle};
use wayland_protocols::xdg::shell::client::xdg_positioner::{
    Anchor, ConstraintAdjustment, Gravity,
};

use super::LayerState;
use crate::event::PopupActivation;

pub(super) struct LayerPopup {
    popup: Popup,
    buffer: Buffer,
    pool: SlotPool,
    pub(super) mapped: bool,
    pub(super) grabbed: bool,
    size: (i32, i32),
    held: PointerButtons,
}

impl LayerPopup {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        anchor: Rect,
        side: LayerEdge,
        frame: LayerPopupFrame,
        panel: Size,
        compositor: &CompositorState,
        shell: &XdgShell,
        shm: &Shm,
        queue: &QueueHandle<LayerState>,
    ) -> Result<Self, LayerError> {
        let (x, y, w, h) = anchor_rect(anchor, panel).ok_or_else(|| {
            LayerError::InvalidSettings("the popup anchor must intersect the panel".into())
        })?;
        let (width, height) = frame.size();
        let width = i32::try_from(width).map_err(LayerError::backend)?;
        let height = i32::try_from(height).map_err(LayerError::backend)?;
        let mut pool = SlotPool::new(frame.pixels().len(), shm).map_err(LayerError::backend)?;
        let buffer = fill_buffer(&frame, &mut pool)?;

        let positioner = XdgPositioner::new(shell).map_err(LayerError::backend)?;
        let (anchor, gravity) = match side {
            LayerEdge::Top => (Anchor::Top, Gravity::Top),
            LayerEdge::Right => (Anchor::Right, Gravity::Right),
            LayerEdge::Bottom => (Anchor::Bottom, Gravity::Bottom),
            LayerEdge::Left => (Anchor::Left, Gravity::Left),
        };
        positioner.set_anchor_rect(x, y, w, h);
        positioner.set_size(width, height);
        positioner.set_anchor(anchor);
        positioner.set_gravity(gravity);
        positioner.set_constraint_adjustment(
            ConstraintAdjustment::FlipX
                | ConstraintAdjustment::FlipY
                | ConstraintAdjustment::SlideX
                | ConstraintAdjustment::SlideY,
        );
        let surface = compositor.create_surface(queue);
        let popup = Popup::from_surface(None, &positioner, queue, surface, shell)
            .map_err(LayerError::backend)?;
        Ok(Self {
            popup,
            buffer,
            pool,
            mapped: false,
            grabbed: false,
            size: (width, height),
            held: PointerButtons::new(),
        })
    }

    pub(super) fn attach(&mut self, parent: &LayerSurface, activation: Option<PopupActivation>) {
        parent.get_popup(self.popup.xdg_popup());
        if let Some(activation) = activation {
            self.popup
                .xdg_popup()
                .grab(&activation.seat, activation.serial);
            self.grabbed = true;
        }
        self.surface().commit();
    }

    pub(super) fn update(&mut self, frame: LayerPopupFrame) -> Result<(), LayerError> {
        let (width, height) = frame.size();
        let size = (
            i32::try_from(width).map_err(LayerError::backend)?,
            i32::try_from(height).map_err(LayerError::backend)?,
        );
        if !self.mapped || size != self.size {
            return Err(LayerError::InvalidSettings(
                "a popup update requires its configured size".into(),
            ));
        }
        let buffer = fill_buffer(&frame, &mut self.pool)?;
        buffer
            .attach_to(self.surface())
            .map_err(LayerError::backend)?;
        self.surface().damage(0, 0, size.0, size.1);
        self.surface().commit();
        self.buffer = buffer;
        Ok(())
    }

    pub(super) fn surface(&self) -> &WlSurface {
        self.popup.wl_surface()
    }
}

#[cfg(test)]
pub(super) fn popup_buffer(frame: &LayerPopupFrame, shm: &Shm) -> Result<Buffer, LayerError> {
    let mut pool = SlotPool::new(frame.pixels().len(), shm).map_err(LayerError::backend)?;
    fill_buffer(frame, &mut pool)
}

fn fill_buffer(frame: &LayerPopupFrame, pool: &mut SlotPool) -> Result<Buffer, LayerError> {
    let (width, height) = frame.size();
    let width = i32::try_from(width).map_err(LayerError::backend)?;
    let height = i32::try_from(height).map_err(LayerError::backend)?;
    let stride_bytes = i32::try_from(frame.row_stride_bytes()).map_err(LayerError::backend)?;
    let (buffer, canvas) = pool
        .create_buffer(width, height, stride_bytes, wl_shm::Format::Argb8888)
        .map_err(LayerError::backend)?;
    for (pixel, output) in frame
        .pixels()
        .chunks_exact(4)
        .zip(canvas.chunks_exact_mut(4))
    {
        let [r, g, b, a] = [pixel[0], pixel[1], pixel[2], pixel[3]];
        output.copy_from_slice(&u32::from_be_bytes([a, r, g, b]).to_ne_bytes());
    }

    Ok(buffer)
}

fn anchor_rect(anchor: Rect, panel: Size) -> Option<(i32, i32, i32, i32)> {
    use super::protocol_coordinate::protocol_coordinate;
    if ![
        anchor.origin.x,
        anchor.origin.y,
        anchor.right(),
        anchor.bottom(),
        panel.width,
        panel.height,
    ]
    .into_iter()
    .all(f32::is_finite)
    {
        return None;
    }
    let left = protocol_coordinate(anchor.origin.x.floor().max(0.0))?;
    let top = protocol_coordinate(anchor.origin.y.floor().max(0.0))?;
    let right = protocol_coordinate(anchor.right().ceil().min(panel.width.floor()))?;
    let bottom = protocol_coordinate(anchor.bottom().ceil().min(panel.height.floor()))?;
    (anchor.size.width > 0.0 && anchor.size.height > 0.0 && right > left && bottom > top)
        .then(|| (left, top, right - left, bottom - top))
}

impl LayerState {
    pub(super) fn is_panel_surface(&self, surface: &WlSurface) -> bool {
        self.layer
            .as_ref()
            .is_some_and(|layer| layer.wl_surface() == surface)
    }

    pub(super) fn dismiss_popup(&mut self) {
        let previous = self.keyboard_route();
        if self.popup.take().is_some() {
            self.popup_keyboard_closed(previous);
            self.popup_events.push(LayerPopupEvent::Closed);
        }
    }

    pub(super) fn popup_touch(
        &mut self,
        surface: &WlSurface,
        id: i32,
        kind: PointerEventKind,
        at: Point,
        time: u32,
    ) {
        if self
            .popup
            .as_ref()
            .is_some_and(|popup| popup.surface() == surface)
        {
            let mut buttons = PointerButtons::new();
            if matches!(kind, PointerEventKind::Down | PointerEventKind::Move) {
                buttons.press(obscur_interaction::pointer::PointerButton::Primary);
            }
            let pointer = PointerEvent::new(
                PointerId::touch(u64::from(id as u32)),
                PointerDevice::Touch,
                kind,
                at,
            )
            .with_buttons(buttons)
            .with_timestamp(u64::from(time) * 1_000)
            .as_trusted();
            self.popup_events.push(LayerPopupEvent::Pointer(pointer));
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn popup_pointer(&mut self, event: &NativePointer) {
        let Some(popup) = self
            .popup
            .as_mut()
            .filter(|popup| popup.surface() == &event.surface)
        else {
            return;
        };
        let (kind, time, button) = match event.kind {
            NativeKind::Enter { .. } => (PointerEventKind::Enter, 0, None),
            NativeKind::Leave { .. } => (PointerEventKind::Leave, 0, None),
            NativeKind::Motion { time } => (PointerEventKind::Move, time, None),
            NativeKind::Press { time, button, .. } => {
                let button = super::layer_state::button_of(button);
                if let Some(button) = button {
                    popup.held.press(button);
                }
                (PointerEventKind::Down, time, button)
            }
            NativeKind::Release { time, button, .. } => {
                let button = super::layer_state::button_of(button);
                if let Some(button) = button {
                    popup.held.release(button);
                }
                (PointerEventKind::Up, time, button)
            }
            NativeKind::Axis {
                horizontal,
                vertical,
                time,
                ..
            } => {
                use smithay_client_toolkit::seat::pointer::AxisScroll;
                let pixels = |axis: AxisScroll| {
                    if axis.absolute == 0.0 {
                        axis.discrete as f32 * 48.0
                    } else {
                        axis.absolute as f32
                    }
                };
                let (x, y) = (pixels(horizontal), pixels(vertical));
                if x != 0.0 || y != 0.0 {
                    let at = Point::new(event.position.0 as f32, event.position.1 as f32);
                    let wheel = WheelEvent::new(at, y)
                        .with_delta_x(x)
                        .with_timestamp(u64::from(time) * 1_000)
                        .as_trusted();
                    self.popup_events.push(LayerPopupEvent::Wheel(wheel));
                }
                return;
            }
        };
        let mut pointer = PointerEvent::new(
            PointerId::MOUSE,
            PointerDevice::Mouse,
            kind,
            Point::new(event.position.0 as f32, event.position.1 as f32),
        )
        .with_buttons(popup.held)
        .with_timestamp(u64::from(time) * 1_000)
        .as_trusted();
        if let Some(button) = button {
            pointer = pointer.with_button(button);
        }
        self.popup_events.push(LayerPopupEvent::Pointer(pointer));
    }
}

impl PopupHandler for LayerState {
    #[allow(clippy::cast_precision_loss)]
    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        popup: &Popup,
        config: PopupConfigure,
    ) {
        let previous_route = self.keyboard_route();
        let Some(active) = self
            .popup
            .as_mut()
            .filter(|active| active.surface() == popup.wl_surface())
        else {
            return;
        };
        let configured_size = (config.width.max(1), config.height.max(1));
        let Some(viewport) = LayerPopupFrame::viewport_for_size(
            configured_size.0.unsigned_abs(),
            configured_size.1.unsigned_abs(),
        ) else {
            self.dismiss_popup();
            self.popup_events.push(LayerPopupEvent::Failed(
                "invalid configured popup dimensions".into(),
            ));
            return;
        };
        let (width, height) = active.size;
        if !active.mapped {
            if let Err(error) = active.buffer.attach_to(popup.wl_surface()) {
                self.dismiss_popup();
                self.popup_events
                    .push(LayerPopupEvent::Failed(error.to_string().into()));
                return;
            }
            popup.wl_surface().damage(0, 0, width, height);
            active.mapped = true;
        }
        active.size = configured_size;
        popup.wl_surface().commit();
        self.popup_events
            .push(LayerPopupEvent::Configured(Rect::new(
                Point::new(config.position.0 as f32, config.position.1 as f32),
                viewport,
            )));
        self.keyboard_route_changed(previous_route);
    }

    fn done(&mut self, _: &Connection, _: &QueueHandle<Self>, popup: &Popup) {
        if self
            .popup
            .as_ref()
            .is_some_and(|active| active.surface() == popup.wl_surface())
        {
            self.dismiss_popup();
        }
    }
}

impl ShmHandler for LayerState {
    fn shm_state(&mut self) -> &mut Shm {
        self.popup_shm
            .as_mut()
            .expect("events are dispatched only for a bound shm global")
    }
}

smithay_client_toolkit::delegate_xdg_shell!(LayerState);
smithay_client_toolkit::delegate_xdg_popup!(LayerState);
smithay_client_toolkit::delegate_shm!(LayerState);

// SCTK's XdgShell also binds the decoration global. This client creates no toplevels.
impl smithay_client_toolkit::shell::xdg::window::WindowHandler for LayerState {
    fn request_close(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::shell::xdg::window::Window,
    ) {
    }

    fn configure(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &smithay_client_toolkit::shell::xdg::window::Window,
        _: smithay_client_toolkit::shell::xdg::window::WindowConfigure,
        _: u32,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_must_intersect_the_panel_and_is_clipped_before_rounding() {
        let panel = Size::new(100.0, 20.0);
        let partially_visible = Rect::new(Point::new(-2.5, 4.5), Size::new(6.0, 20.0));
        assert_eq!(anchor_rect(partially_visible, panel), Some((0, 4, 4, 16)));
        let invisible = Rect::new(Point::new(101.0, 5.0), Size::new(3.0, 3.0));
        assert_eq!(anchor_rect(invisible, panel), None);
        let invalid = Rect::new(Point::new(f32::NAN, 5.0), Size::new(3.0, 3.0));
        assert_eq!(anchor_rect(invalid, panel), None);
        let oversized = Rect::new(Point::ORIGIN, Size::new(2_147_483_648.0, 20.0));
        assert_eq!(anchor_rect(oversized, oversized.size), None);
        assert_eq!(anchor_rect(oversized, panel), Some((0, 0, 100, 20)));
    }
}
