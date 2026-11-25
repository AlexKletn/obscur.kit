//! A live subscription to the desktop's appearance settings.

#[doc = include_str!("../../docs/api/appearance/appearance_watch.doc/AppearanceWatch.md")]
pub struct AppearanceWatch {
    stop: Option<Box<dyn FnOnce() + Send>>,
}

impl AppearanceWatch {
    /// A watch that runs this when it ends.
    #[must_use]
    pub fn new(stop: impl FnOnce() + Send + 'static) -> Self {
        Self {
            stop: Some(Box::new(stop)),
        }
    }

    #[doc = include_str!("../../docs/api/appearance/appearance_watch.doc/inert.md")]
    #[must_use]
    pub fn inert() -> Self {
        Self { stop: None }
    }

    /// End the watch now rather than when it goes out of scope.
    pub fn stop(mut self) {
        self.run_stop();
    }

    /// Run the teardown at most once, whichever way the end arrives.
    fn run_stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            stop();
        }
    }
}

impl Drop for AppearanceWatch {
    fn drop(&mut self) {
        self.run_stop();
    }
}

impl std::fmt::Debug for AppearanceWatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppearanceWatch")
            .field("live", &self.stop.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn dropping_the_guard_stops_the_watch() {
        // The whole contract: an application that forgets to call `stop` still
        // does not leak a thread or a child process.
        let stopped = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&stopped);
        drop(AppearanceWatch::new(move || {
            seen.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(stopped.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn stopping_explicitly_does_not_then_stop_again_on_drop() {
        let stopped = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&stopped);
        AppearanceWatch::new(move || {
            seen.fetch_add(1, Ordering::SeqCst);
        })
        .stop();
        assert_eq!(stopped.load(Ordering::SeqCst), 1, "exactly once");
    }

    #[test]
    fn an_inert_watch_is_a_watch_that_does_nothing() {
        // What a source returns when it has nothing to tear down but the caller
        // still holds a guard of the same type.
        let watch = AppearanceWatch::inert();
        assert!(format!("{watch:?}").contains("live: false"));
        drop(watch);
    }
}
