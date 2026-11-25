//! Independent runtime and CPU rasterization for views hosted in layer popups.

use crate::bootstrap::{CssPipeline, InstalledFonts, KitBootstrap};
use crate::identity::{RuntimeId, VersionStamp};
use crate::render::{
    Renderer,
    frame::{FrameSubmission, RenderFrame},
    target::{BitmapTarget, RenderTarget},
};
use crate::render_cpu::renderer::CpuRenderer;
use crate::runtime::{UiRuntime, frame::FrameReason};
use crate::windowing::layer::{LayerPopupEvent, LayerPopupFrame};

use super::popup_content::PopupContent;

pub(super) struct LayerPopupUi {
    runtime: UiRuntime,
    renderer: CpuRenderer,
    fonts: InstalledFonts,
    sheets: Vec<VersionStamp>,
    ready: bool,
}

impl LayerPopupUi {
    pub(super) fn open(
        boot: &KitBootstrap,
        css: &CssPipeline,
        size: (u32, u32),
        content: PopupContent,
        media_input: Option<crate::style::matching::MediaInput>,
    ) -> Result<(Self, LayerPopupFrame), Box<str>> {
        let viewport =
            LayerPopupFrame::viewport_for_size(size.0, size.1).ok_or("invalid popup dimensions")?;
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(2), viewport);
        runtime.set_detected_media_input(media_input);
        boot.install_default_stylesheet(&mut runtime);
        let _ = boot.install_builtin_components(&mut runtime);
        boot.install_builtin_painters(&mut runtime);
        #[cfg(all(feature = "resource-fs", feature = "resource-decode"))]
        {
            boot.install_image_resources(&mut runtime, ".");
            boot.install_vector_resources(&mut runtime);
        }
        #[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
        let fonts = boot.install_text_stack(&mut runtime, &boot.font_set());
        #[cfg(not(all(feature = "text-harfrust", feature = "text-unicode")))]
        let fonts = InstalledFonts::default();
        let mut renderer = boot.default_renderer();
        for (handle, bytes) in fonts.iter() {
            renderer.register_font(handle, bytes.to_vec());
        }

        css.install_on(&mut runtime);
        runtime.set_root_view(content.build);
        (content.configure)(&mut runtime);
        runtime.request_frame(FrameReason::UiCommit);
        let mut ui = Self {
            runtime,
            renderer,
            fonts,
            sheets: css.sheets().iter().map(|sheet| sheet.stamp()).collect(),
            ready: false,
        };
        ui.runtime.run_due_frame_reactive();
        let first = ui.rasterize()?;
        Ok((ui, first))
    }

    pub(super) fn handle_event(&mut self, event: &LayerPopupEvent) {
        match event {
            LayerPopupEvent::Configured(rect) => {
                self.ready = true;
                if self.runtime.viewport() != rect.size {
                    self.runtime.set_viewport(rect.size);
                    self.runtime.request_frame(FrameReason::WindowChange);
                }
            }
            LayerPopupEvent::Pointer(pointer) => {
                self.runtime.handle_pointer(*pointer);
            }
            LayerPopupEvent::Wheel(wheel) => {
                self.runtime.handle_wheel(*wheel);
            }
            LayerPopupEvent::Key(key) => {
                self.runtime.handle_key(*key);
            }
            LayerPopupEvent::Modifiers(modifiers) => self.runtime.set_modifiers(*modifiers),
            LayerPopupEvent::KeyboardFocus(true) => {
                self.runtime.focus_first_window_tab_stop();
            }
            LayerPopupEvent::KeyboardFocus(false) => self.runtime.blur_window_focus(),
            LayerPopupEvent::Closed | LayerPopupEvent::Failed(_) => {}
        }
    }

    pub(super) fn request_frame(&mut self) {
        self.runtime.request_frame(FrameReason::UiCommit);
    }

    pub(super) fn pending(&self) -> bool {
        self.ready && self.runtime.has_pending_frame()
    }

    pub(super) fn set_detected_media_input(
        &mut self,
        input: Option<crate::style::matching::MediaInput>,
    ) {
        self.runtime.set_detected_media_input(input);
    }

    pub(super) fn tick(
        &mut self,
        css: &CssPipeline,
        delta: f32,
    ) -> Result<Option<LayerPopupFrame>, Box<str>> {
        if !self.ready {
            return Ok(None);
        }
        if css
            .sheets()
            .iter()
            .map(|sheet| sheet.stamp())
            .ne(self.sheets.iter().copied())
        {
            css.install_on(&mut self.runtime);
            self.sheets = css.sheets().iter().map(|sheet| sheet.stamp()).collect();
        }
        self.runtime.advance_motion(delta);
        self.runtime.poll_resource_tasks();
        if self.runtime.run_due_frame_reactive().is_none() {
            return Ok(None);
        }
        let frame = self.rasterize()?;
        self.runtime.poll_resource_tasks();
        Ok(Some(frame))
    }

    fn rasterize(&mut self) -> Result<LayerPopupFrame, Box<str>> {
        for (handle, bytes) in self.fonts.take_discovered() {
            self.renderer.register_font(handle, bytes.to_vec());
        }
        let size = self.runtime.viewport();
        let submission = FrameSubmission::new(RenderFrame::new(self.runtime.snapshot(), size));
        self.renderer
            .render(&submission, &RenderTarget::bitmap(BitmapTarget::new(size)))
            .map_err(|error| error.to_string().into_boxed_str())?;
        let bitmap = self
            .renderer
            .pixmap()
            .ok_or("the popup renderer returned no pixels")?;
        LayerPopupFrame::new(bitmap.width(), bitmap.height(), bitmap.data().to_vec())
            .ok_or_else(|| "the popup renderer returned an invalid frame".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::source::CssSource;
    use crate::geometry::{Point, Rect, Size};
    use crate::identity::{ComponentId, InternedSymbol};
    use crate::interaction::pointer::{
        PointerButton, PointerButtons, PointerDevice, PointerEvent, PointerEventKind, PointerId,
    };
    use crate::view::view::{
        CLASS, EVENT_CLICK, View, ViewEventBinding, ViewNode, ViewProperty, ViewValue,
    };
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn a_popup_click_rebuilds_and_repaints_its_own_view() {
        let boot = KitBootstrap::new();
        let mut css = CssPipeline::new();
        let mut panel = UiRuntime::new(RuntimeId::from_raw(1), Size::new(128.0, 32.0));
        css.install(
            &mut panel,
            &[CssSource::anonymous(
                ".red { width: 32px; height: 24px; background-color: red; }
             .blue { width: 32px; height: 24px; background-color: blue; }",
            )],
        );

        let command = InternedSymbol::named("popup-test-command");
        let clicked = Rc::new(Cell::new(false));
        let build_state = Rc::clone(&clicked);
        let command_state = Rc::clone(&clicked);
        let content = PopupContent {
            build: Box::new(move || {
                let class = if build_state.get() { "blue" } else { "red" };
                View::single(
                    ViewNode::element(ComponentId::from_raw(1))
                        .with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))
                        .with_event(ViewEventBinding::new(EVENT_CLICK, command)),
                )
            }),
            configure: Box::new(move |runtime| {
                runtime.on_command(command, move || command_state.set(true))
            }),
        };
        let (mut ui, first) = LayerPopupUi::open(&boot, &css, (32, 24), content, None).unwrap();
        assert_eq!(&first.pixels()[0..4], &[255, 0, 0, 255]);
        assert!(ui.tick(&css, 0.0).unwrap().is_none());
        ui.handle_event(&LayerPopupEvent::Configured(Rect::new(
            Point::ORIGIN,
            Size::new(32.0, 24.0),
        )));

        let mut held = PointerButtons::new();
        held.press(PointerButton::Primary);
        for (kind, buttons) in [
            (PointerEventKind::Down, held),
            (PointerEventKind::Up, PointerButtons::new()),
        ] {
            let event = PointerEvent::new(
                PointerId::MOUSE,
                PointerDevice::Mouse,
                kind,
                Point::new(5.0, 5.0),
            )
            .with_buttons(buttons)
            .with_button(PointerButton::Primary)
            .as_trusted();
            ui.handle_event(&LayerPopupEvent::Pointer(event));
        }
        assert!(clicked.get());
        assert!(panel.focused().is_none());
        let next = ui.tick(&css, 0.0).unwrap().unwrap();
        assert_eq!(&next.pixels()[0..4], &[0, 0, 255, 255]);
    }
    #[test]
    fn native_keyboard_focus_and_enter_activate_the_popup_button() {
        use crate::interaction::command::{KeyEvent, LogicalKey, NamedKey, PhysicalKey};

        let boot = KitBootstrap::new();
        let css = CssPipeline::new();
        let command = InternedSymbol::named("popup-keyboard-command");
        let fired = Rc::new(Cell::new(false));
        let handler = Rc::clone(&fired);
        let content = PopupContent {
            build: Box::new(move || crate::widgets::button::Button::new("Open", command).view()),
            configure: Box::new(move |runtime| {
                runtime.on_command(command, move || handler.set(true))
            }),
        };
        let (mut ui, _) = LayerPopupUi::open(&boot, &css, (120, 42), content, None).unwrap();
        ui.handle_event(&LayerPopupEvent::KeyboardFocus(true));
        assert!(ui.runtime.focused().is_some());
        ui.handle_event(&LayerPopupEvent::Key(
            KeyEvent::press(PhysicalKey::Enter, Some(LogicalKey::named(NamedKey::Enter)))
                .as_trusted(),
        ));
        assert!(fired.get());
        ui.handle_event(&LayerPopupEvent::KeyboardFocus(false));
        assert!(ui.runtime.focused().is_none());
    }
}
