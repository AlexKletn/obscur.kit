//! Wayland text-input v3 state for editable panel fields.

use obscur_geometry::Rect;
use obscur_identity::NodeId;
use obscur_text::{
    ime::{Preedit, TextInputUpdate},
    span::TextRange,
};
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
};
use wayland_protocols::wp::text_input::zv3::client::{
    zwp_text_input_manager_v3::ZwpTextInputManagerV3,
    zwp_text_input_v3::{self, ZwpTextInputV3},
};

use super::input_context::InputContext;
use super::{LayerState, layer_keyboard::KeyboardTarget};
use crate::event::LayerInput;

#[derive(Default)]
pub(super) struct LayerIme {
    pub(super) manager: Option<ZwpTextInputManagerV3>,
    input: Option<ZwpTextInputV3>,
    focused: Option<WlSurface>,
    context: Option<InputContext>,
    pending: TextInputUpdate,
    serial: u32,
    enabled_at: u32,
    can_sync: bool,
}

impl LayerIme {
    pub(super) fn bind_seat(&mut self, seat: &WlSeat, qh: &QueueHandle<LayerState>) {
        if let Some(manager) = &self.manager {
            self.input = Some(manager.get_text_input(seat, qh, ()));
            self.can_sync = true;
        }
    }

    pub(super) fn remove_seat(&mut self) {
        if let Some(input) = self.input.take() {
            input.destroy();
        }
        self.clear_focus();
        self.serial = 0;
        self.enabled_at = 0;
    }

    fn clear_focus(&mut self) {
        self.focused = None;
        self.context = None;
        self.pending = TextInputUpdate::default();
        self.can_sync = true;
    }

    fn commit(&mut self, input: &ZwpTextInputV3) {
        input.commit();
        self.serial = self.serial.wrapping_add(1);
    }

    pub(super) fn sync(&mut self, field: Option<(NodeId, &str, usize, usize, Option<Rect>)>) {
        let Some(input) = self.input.clone().filter(|_| self.focused.is_some()) else {
            return;
        };
        let next = field.map(|(node, text, cursor, anchor, rect)| InputContext {
            node,
            surrounding: surrounding(text, cursor, anchor),
            rect: rect.and_then(cursor_rectangle),
        });
        if next == self.context {
            return;
        }
        let switching =
            self.context.as_ref().map(|ctx| ctx.node) != next.as_ref().map(|ctx| ctx.node);
        if switching {
            if self.context.is_some() {
                input.disable();
                self.commit(&input);
            }
            self.pending = TextInputUpdate::default();
            self.can_sync = true;
            if next.is_some() {
                input.enable();
                input.set_content_type(
                    zwp_text_input_v3::ContentHint::None,
                    zwp_text_input_v3::ContentPurpose::Normal,
                );
                self.enabled_at = self.serial.wrapping_add(1);
            }
        } else if !self.can_sync {
            return;
        }
        if let Some(next) = &next {
            if let Some((text, cursor, anchor)) = &next.surrounding {
                input.set_surrounding_text(text.clone(), *cursor, *anchor);
            } else if !switching
                && self
                    .context
                    .as_ref()
                    .is_some_and(|current| current.surrounding.is_some())
            {
                input.set_surrounding_text(String::new(), 0, 0);
            }
            if let Some((x, y, width, height)) = next.rect {
                input.set_cursor_rectangle(x, y, width, height);
            }
            self.commit(&input);
        }
        self.context = next;
    }
}

fn surrounding(text: &str, cursor: usize, anchor: usize) -> Option<(String, i32, i32)> {
    const MAX_BYTES: usize = 4000;
    let (low, high) = (cursor.min(anchor), cursor.max(anchor));
    if !text.is_char_boundary(low)
        || !text.is_char_boundary(high)
        || high - low > MAX_BYTES
        || text.contains('\0')
    {
        return None;
    }
    let mut start = low.saturating_sub((MAX_BYTES - (high - low)) / 2);
    let mut end = start.saturating_add(MAX_BYTES).min(text.len());
    start = end.saturating_sub(MAX_BYTES);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let cursor_bytes = i32::try_from(cursor.checked_sub(start)?).ok()?;
    let anchor_bytes = i32::try_from(anchor.checked_sub(start)?).ok()?;
    Some((text.get(start..end)?.to_owned(), cursor_bytes, anchor_bytes))
}

fn cursor_rectangle(rect: Rect) -> Option<(i32, i32, i32, i32)> {
    use super::protocol_coordinate::protocol_coordinate;
    if ![
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
    ]
    .into_iter()
    .all(f32::is_finite)
    {
        return None;
    }
    Some((
        protocol_coordinate(rect.origin.x.floor())?,
        protocol_coordinate(rect.origin.y.floor())?,
        protocol_coordinate(rect.size.width.ceil().max(1.0))?,
        protocol_coordinate(rect.size.height.ceil().max(1.0))?,
    ))
}

fn preedit(text: String, begin: i32, end: i32) -> Preedit {
    let mut preedit = Preedit::new(text);
    if let Ok(begin) = usize::try_from(begin) {
        preedit = preedit.with_caret(begin);
        if let Ok(end) = usize::try_from(end)
            && begin != end
            && preedit.text().is_char_boundary(begin)
            && preedit.text().is_char_boundary(end)
        {
            preedit = preedit.with_highlight(TextRange::new(begin.min(end), begin.max(end)));
        }
    }
    preedit
}

impl Dispatch<ZwpTextInputV3, ()> for LayerState {
    fn event(
        state: &mut Self,
        input: &ZwpTextInputV3,
        event: zwp_text_input_v3::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if state.ime.input.as_ref() != Some(input) {
            return;
        }
        match event {
            zwp_text_input_v3::Event::Enter { surface } => {
                state.ime.clear_focus();
                if state.is_panel_surface(&surface) {
                    state.ime.focused = Some(surface);
                }
            }
            zwp_text_input_v3::Event::Leave { surface }
                if state.ime.focused.as_ref() == Some(&surface) =>
            {
                let target = state.ime.context.as_ref().map(|ctx| ctx.node);
                state.ime.clear_focus();
                if let Some(node) = target {
                    state.push_input(LayerInput::TextInput(node, TextInputUpdate::default()));
                }
            }
            zwp_text_input_v3::Event::PreeditString {
                text,
                cursor_begin,
                cursor_end,
            } => {
                state.ime.pending.preedit = text
                    .filter(|text| !text.is_empty())
                    .map(|text| preedit(text, cursor_begin, cursor_end));
            }
            zwp_text_input_v3::Event::CommitString { text } => {
                state.ime.pending.commit = text.map(String::into_boxed_str);
            }
            zwp_text_input_v3::Event::DeleteSurroundingText {
                before_length,
                after_length,
            } => {
                state.ime.pending.before = before_length;
                state.ime.pending.after = after_length;
            }
            zwp_text_input_v3::Event::Done { serial } => {
                let pending = core::mem::take(&mut state.ime.pending);
                state.ime.can_sync = serial == state.ime.serial;
                if state.keyboard_route() == Some(KeyboardTarget::Panel)
                    && serial.wrapping_sub(state.ime.enabled_at) < (1 << 31)
                    && let Some(node) = state.ime.context.as_ref().map(|ctx| ctx.node)
                {
                    state.push_input(LayerInput::TextInput(node, pending));
                }
            }
            _ => {}
        }
    }
}

wayland_client::delegate_noop!(LayerState: ignore ZwpTextInputManagerV3);

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_geometry::{Point, Size};

    #[test]
    fn cursor_rectangle_rounds_in_range_and_rejects_overflow() {
        let size = Size::new(0.0, 2.25);
        assert_eq!(
            cursor_rectangle(Rect::new(Point::new(-1.25, 4.75), size)),
            Some((-2, 4, 1, 3))
        );
        assert_eq!(
            cursor_rectangle(Rect::new(Point::new(2_147_483_648.0, 0.0), size)),
            None
        );
        assert_eq!(
            cursor_rectangle(Rect::new(Point::ORIGIN, Size::new(1.0, f32::INFINITY))),
            None
        );
    }

    #[test]
    fn surrounding_keeps_a_backward_selection_in_a_utf8_window() {
        let text = "界".repeat(4_000);
        let (window, cursor, anchor) = surrounding(&text, 6_000, 6_009).unwrap();
        assert!(window.len() <= 4_000);
        let selected = usize::try_from(cursor).unwrap()..usize::try_from(anchor).unwrap();
        assert_eq!(window.get(selected), Some("界界界"));
        assert!(surrounding(&text, 0, 6_000).is_none());
        assert!(surrounding(&text, 1, 3).is_none());
    }
}
