//! Coalesces appearance notifications and applies snapshots on the application event loop.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::bootstrap::KitBootstrap;
use crate::runtime::UiRuntime;
use crate::runtime::frame::FrameReason;
use crate::system::appearance::{
    AppearanceCallback, AppearanceProvider, AppearanceWatch, SystemAppearance,
};

/// Owns the provider and its watch for the lifetime of the application.
pub(super) struct AppearanceSubscription {
    // Stop notifications before dropping their provider.
    watch: Option<AppearanceWatch>,
    provider: Box<dyn AppearanceProvider>,
    snapshot: SystemAppearance,
    dirty: Arc<AtomicBool>,
}

impl AppearanceSubscription {
    /// Retains the provider and the snapshot already used at startup.
    pub(super) fn new(provider: Box<dyn AppearanceProvider>, snapshot: SystemAppearance) -> Self {
        Self {
            watch: None,
            provider,
            snapshot,
            dirty: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Subscribes to provider changes and schedules a catch-up read.
    pub(super) fn start(&mut self, wake: impl Fn() + Send + Sync + 'static) {
        let dirty = Arc::clone(&self.dirty);
        let notify: AppearanceCallback = Arc::new(move || {
            if !dirty.swap(true, Ordering::AcqRel) {
                wake();
            }
        });
        self.watch = self.provider.watch(Arc::clone(&notify));
        if self.watch.is_some() {
            // An update can arrive between the boot snapshot and subscription.
            notify();
        }
    }

    /// Applies the latest pending change on the application event loop.
    pub(super) fn refresh(&mut self, runtime: &mut UiRuntime) {
        if !self.dirty.swap(false, Ordering::AcqRel) {
            return;
        }
        let snapshot = self.provider.appearance();
        if snapshot == self.snapshot {
            return;
        }
        KitBootstrap::new().install_system_theme(runtime, &snapshot);
        self.snapshot = snapshot;
        runtime.request_frame(FrameReason::UiCommit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::SupportLevel;
    use crate::geometry::Size;
    use crate::identity::RuntimeId;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;

    struct Provider {
        snapshot: Arc<Mutex<SystemAppearance>>,
        callback: Arc<Mutex<Option<AppearanceCallback>>>,
    }

    impl AppearanceProvider for Provider {
        fn appearance(&self) -> SystemAppearance {
            self.snapshot.lock().unwrap().clone()
        }

        fn support(&self) -> SupportLevel {
            SupportLevel::Native
        }

        fn watch(&self, callback: AppearanceCallback) -> Option<AppearanceWatch> {
            *self.callback.lock().unwrap() = Some(callback);
            let slot = Arc::clone(&self.callback);
            Some(AppearanceWatch::new(move || {
                slot.lock().unwrap().take();
            }))
        }
    }

    #[test]
    fn notifications_coalesce_and_refresh_on_the_app_thread() {
        let callback = Arc::new(Mutex::new(None));
        let snapshot = Arc::new(Mutex::new(SystemAppearance::default()));
        let provider = Provider {
            snapshot: Arc::clone(&snapshot),
            callback: Arc::clone(&callback),
        };
        let wakes = Arc::new(AtomicUsize::new(0));
        let mut subscription =
            AppearanceSubscription::new(Box::new(provider), SystemAppearance::default());
        let observed = Arc::clone(&wakes);
        subscription.start(move || {
            observed.fetch_add(1, Ordering::Relaxed);
        });

        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(100.0, 100.0));
        subscription.refresh(&mut runtime);
        assert_eq!(wakes.load(Ordering::Relaxed), 1);

        let notify = callback.lock().unwrap().clone().unwrap();
        std::thread::spawn(move || {
            for radius in [12.0, 24.0] {
                snapshot.lock().unwrap().window_radius = Some(radius);
                notify();
            }
        })
        .join()
        .unwrap();
        assert_eq!(wakes.load(Ordering::Relaxed), 2);
        assert_eq!(subscription.snapshot.window_radius, None);

        subscription.refresh(&mut runtime);
        assert_eq!(subscription.snapshot.window_radius, Some(24.0));
        assert!(runtime.has_pending_frame());

        drop(subscription);
        assert!(callback.lock().unwrap().is_none());
    }
}
