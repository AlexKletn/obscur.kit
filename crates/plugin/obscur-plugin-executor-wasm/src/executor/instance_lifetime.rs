use std::cell::{Cell, RefCell};

use event_listener::Event;
use obscur_plugin_api::status::PluginApiError;

#[derive(Default)]
pub(super) struct InstanceLifetime {
    closed: Cell<bool>,
    suspended: Cell<bool>,
    ended: Event,
    fault: RefCell<Option<PluginApiError>>,
}

impl InstanceLifetime {
    pub(super) fn is_closed(&self) -> bool {
        self.closed.get()
    }

    pub(super) fn is_suspended(&self) -> bool {
        self.suspended.get()
    }

    pub(super) fn set_suspended(&self, suspended: bool) {
        self.suspended.set(suspended);
    }

    pub(super) async fn wait_closed(&self) {
        let listener = self.ended.listen();
        if !self.is_closed() {
            listener.await;
        }
    }

    pub(super) fn fault(&self) -> Option<PluginApiError> {
        self.fault.borrow().clone()
    }

    pub(super) fn is_faulted(&self) -> bool {
        self.fault.borrow().is_some()
    }

    pub(super) fn record_fault(&self, error: PluginApiError) {
        self.fault.replace(Some(error));
        self.close();
    }

    pub(super) fn close(&self) {
        if !self.closed.replace(true) {
            self.ended.notify(usize::MAX);
        }
    }
}
