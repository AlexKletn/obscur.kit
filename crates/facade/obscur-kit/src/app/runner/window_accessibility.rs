use std::{cell::RefCell, rc::Rc};

use accesskit_winit::{Adapter, WindowEvent as AccessibilityEvent};
use obscur_accessibility_accesskit::bridge::AccessKitBridge;
use obscur_runtime::UiRuntime;
use winit::{event::WindowEvent, event_loop::EventLoopProxy, window::Window};

use super::WindowWake;

pub(super) struct WindowAccessibility {
    adapter: Adapter,
    bridge: Rc<RefCell<AccessKitBridge>>,
    published: Option<u64>,
}

impl WindowAccessibility {
    pub(super) fn new(
        window: &Window,
        proxy: EventLoopProxy<WindowWake>,
        runtime: &mut UiRuntime,
    ) -> Self {
        let adapter = Adapter::with_event_loop_proxy(window, proxy);
        let bridge = crate::bootstrap::KitBootstrap::new().install_accessibility(runtime);
        Self {
            adapter,
            bridge,
            published: None,
        }
    }

    pub(super) fn process_event(&mut self, window: &Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
    }

    pub(super) fn handle_event(&mut self, event: AccessibilityEvent) {
        match event {
            AccessibilityEvent::InitialTreeRequested => {
                self.published = None;
                self.sync();
            }
            AccessibilityEvent::ActionRequested(request) => {
                self.bridge.borrow_mut().push_action_request(request);
            }
            AccessibilityEvent::AccessibilityDeactivated => self.published = None,
        }
    }

    pub(super) fn sync(&mut self) {
        let bridge = self.bridge.borrow();
        let revision = bridge.tree_revision();
        if self.published != Some(revision)
            && let Some(update) = bridge.last_tree_update()
        {
            self.adapter.update_if_active(|| {
                self.published = Some(revision);
                update.clone()
            });
        }
    }
}
