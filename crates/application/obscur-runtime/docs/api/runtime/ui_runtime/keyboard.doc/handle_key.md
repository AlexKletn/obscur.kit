Routes a keyboard [`KeyEvent`] to the currently focused node, resolves it
against the bound command shortcuts, and schedules a frame — the keyboard
analogue of [`handle_pointer`](Self::handle_pointer) (spec §6.7, §3.10).

The flow mirrors the pointer path:

1. **Modifier tracking** — a physical modifier key
   (`Shift`/`Control`/`Alt`/`Meta`) folds into the runtime's held
   [`Modifiers`] set (a press adds, a release removes) so a later shortcut
   sees the live chord. Modifier keys still route like any other key.
2. **Route** — the event is aimed at the [`focused`](Self::focused) node.
   With nothing focused the key routes nowhere: the method is a no-op that
   returns [`KeyDispatch::unhandled`] and schedules no frame.
3. **Resolve + dispatch** — on a **press**, the event's layout-dependent
   [`LogicalKey`](obscur_interaction::command::LogicalKey) plus the held
   modifiers are looked up in the [`CommandRouter`]; a match yields the
   bound command [`InternedSymbol`], whose registered handler is
   **run** via the same [`dispatch_command`](Self::dispatch_command) path
   a pointer click uses. A release, an event with no logical key, or an
   unbound chord resolves to (and dispatches) no command.

   3b. **Activate** — *in addition*, a press whose logical key is Enter or
   Space [`activate`](Self::activate)s the focused control, so a plain
   Enter/Space with no bound shortcut still fires the focused node's
   `on-click` (button) or `on-change` (checkbox) command. Both steps can
   apply on one keypress.

4. **Schedule** — because the key reached a focused node, a
   [`FrameReason::Input`] frame is requested so any focus/command-driven
   visual reaches paint on the next `run_due_frame`/`render_now`.

Returns a [`KeyDispatch`] describing the routed target and any resolved
(and dispatched) shortcut command.

[`Modifiers`]: obscur_interaction::command::Modifiers
[`InternedSymbol`]: obscur_identity::InternedSymbol
[`CommandRouter`]: obscur_interaction::command::CommandRouter
