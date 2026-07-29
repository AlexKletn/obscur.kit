//! The [`AppCommands`] application-level command fallback (spec §2.6, link 5).

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use obscur_identity::InternedSymbol;
use obscur_interaction::event::UiEvent;

use super::{CommandHandler, IntoCommandHandler};

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
/// Handlers take either shape [`IntoCommandHandler`] accepts — `FnMut()` or
/// `FnMut(&mut UiEvent)` — and are not `Send`, matching the rest of the runtime's
/// single-threaded handler model.
#[derive(Clone, Default)]
pub struct AppCommands {
    handlers: Rc<RefCell<HashMap<InternedSymbol, CommandHandler>>>,
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
    ///
    /// `handler` may take nothing or take the propagating [`UiEvent`] — see
    /// [`IntoCommandHandler`]. An application-level fallback is the link most
    /// likely to want the event, because it is the one link that has no idea
    /// *where* the command came from until it reads
    /// [`UiEvent::target`].
    pub fn on_command<M>(&self, command: InternedSymbol, handler: impl IntoCommandHandler<M>) {
        self.handlers
            .borrow_mut()
            .insert(command, handler.into_command_handler());
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

    /// Runs the fallback registered for `command` against the propagating
    /// `event`, returning whether one ran.
    ///
    /// Called by the runtime as the final link of the routing chain. Whatever the
    /// handler sets on `event` — `prevent_default`, `stop_propagation` — is
    /// visible to the caller afterwards, exactly as for the other four links.
    pub(crate) fn dispatch(&self, command: InternedSymbol, event: &mut UiEvent) -> bool {
        // The borrow is released before returning; a handler cannot re-enter
        // this table because it is a closure over app state with no handle to the
        // runtime.
        let mut handlers = self.handlers.borrow_mut();
        match handlers.get_mut(&command) {
            Some(handler) => {
                handler.call(event);
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
    use obscur_identity::NodeId;
    use obscur_interaction::event::EventOrigin;
    use std::cell::Cell;

    const CMD: InternedSymbol = InternedSymbol::from_raw(0x11);

    fn event() -> UiEvent {
        UiEvent::new(EventOrigin::Keyboard, NodeId::from_raw(3))
    }

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

        assert!(app.dispatch(CMD, &mut event()));
        assert_eq!(ran.get(), 1);
        assert!(!app.dispatch(InternedSymbol::from_raw(0x99), &mut event()));
        assert_eq!(ran.get(), 1);

        assert!(app.remove(CMD));
        assert!(!app.dispatch(CMD, &mut event()));
        assert_eq!(ran.get(), 1);
    }

    /// §2.6's last link is the one with no positional information of its own, so
    /// it is the one that most needs the event: before this, an application
    /// fallback could not tell which node the command came from.
    #[test]
    fn an_application_fallback_can_read_the_event_it_was_given() {
        let app = AppCommands::new();
        let seen = Rc::new(Cell::new(None));
        let sink = Rc::clone(&seen);
        app.on_command(CMD, move |e: &mut UiEvent| {
            sink.set(e.target());
            e.prevent_default();
        });

        let mut dispatched = event();
        assert!(app.dispatch(CMD, &mut dispatched));

        assert_eq!(seen.get(), Some(NodeId::from_raw(3)));
        assert!(
            dispatched.is_default_prevented(),
            "what the fallback sets is visible to the runtime that called it"
        );
    }
}
