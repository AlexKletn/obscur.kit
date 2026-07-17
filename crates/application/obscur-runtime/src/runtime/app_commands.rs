//! The [`AppCommands`] application-level command fallback (spec §2.6, link 5).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use obscur_identity::InternedSymbol;

/// The **application** level of §2.6's command routing chain — its last link.
///
/// A [`UiRuntime`](super::UiRuntime) is one *window surface*: it owns one tree,
/// one viewport and one focus. So a handler registered on the runtime itself
/// ([`on_command`](super::UiRuntime::on_command)) is a **window**-level handler
/// — link 4 — and cannot also serve as §2.6's *application* fallback, which by
/// definition outlives and spans windows.
///
/// `AppCommands` is that level: a registry held behind a shared handle, so the
/// **same** fallback can be installed on every window's runtime via
/// [`set_app_commands`](super::UiRuntime::set_app_commands). Cloning an
/// `AppCommands` shares the underlying table rather than copying it — a command
/// registered once is then the fallback for every window that shares it, which
/// is what makes the window and application links genuinely distinct rather than
/// two maps consulted in sequence.
///
/// Handlers are `FnMut()` and are not `Send`, matching the rest of the runtime's
/// single-threaded handler model.
#[derive(Clone, Default)]
pub struct AppCommands {
    handlers: Rc<RefCell<HashMap<InternedSymbol, Box<dyn FnMut()>>>>,
}

impl AppCommands {
    /// Creates an empty application-level fallback registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `handler` as the application fallback for `command`, replacing
    /// any handler previously registered for that symbol.
    ///
    /// The handler runs only when no window-level handler, and nothing in the
    /// focused node's chain, claimed the command first (spec §2.6).
    pub fn on_command(&self, command: InternedSymbol, handler: impl FnMut() + 'static) {
        self.handlers
            .borrow_mut()
            .insert(command, Box::new(handler));
    }

    /// Removes the application fallback for `command`, returning whether one was
    /// registered.
    pub fn remove(&self, command: InternedSymbol) -> bool {
        self.handlers.borrow_mut().remove(&command).is_some()
    }

    /// Returns whether an application fallback is registered for `command`.
    #[must_use]
    pub fn contains(&self, command: InternedSymbol) -> bool {
        self.handlers.borrow().contains_key(&command)
    }

    /// Returns whether two handles share the same underlying registry.
    ///
    /// True exactly when one was [`Clone`]d from the other — the property that
    /// makes an `AppCommands` an *application* level spanning several windows,
    /// rather than a second per-window table.
    #[must_use]
    pub fn shares_registry_with(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.handlers, &other.handlers)
    }

    /// Runs the fallback registered for `command`, returning whether one ran.
    ///
    /// Called by the runtime as the final link of the routing chain.
    pub(crate) fn dispatch(&self, command: InternedSymbol) -> bool {
        // The borrow is released before returning; a handler cannot re-enter
        // this table because it is an `FnMut()` closure over app state with no
        // handle to the runtime.
        let mut handlers = self.handlers.borrow_mut();
        match handlers.get_mut(&command) {
            Some(handler) => {
                handler();
                true
            }
            None => false,
        }
    }
}

impl std::fmt::Debug for AppCommands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppCommands")
            .field("commands", &self.handlers.borrow().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    const CMD: InternedSymbol = InternedSymbol::from_raw(0x11);

    #[test]
    fn a_clone_shares_the_registry_and_a_fresh_one_does_not() {
        let app = AppCommands::new();
        let shared = app.clone();
        let separate = AppCommands::new();

        assert!(app.shares_registry_with(&shared));
        assert!(!app.shares_registry_with(&separate));

        // Registering through one handle is visible through its clone.
        app.on_command(CMD, || {});
        assert!(shared.contains(CMD));
        assert!(!separate.contains(CMD));
    }

    #[test]
    fn dispatch_runs_the_handler_once_and_reports_a_miss() {
        let app = AppCommands::new();
        let ran = Rc::new(Cell::new(0_u32));
        let counter = Rc::clone(&ran);
        app.on_command(CMD, move || counter.set(counter.get() + 1));

        assert!(app.dispatch(CMD));
        assert_eq!(ran.get(), 1);
        assert!(!app.dispatch(InternedSymbol::from_raw(0x99)));
        assert_eq!(ran.get(), 1);

        assert!(app.remove(CMD));
        assert!(!app.dispatch(CMD));
        assert_eq!(ran.get(), 1);
    }
}
