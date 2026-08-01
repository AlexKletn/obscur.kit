Registers `handler` as the **window**-level logic to run when `command`
is dispatched, replacing any handler previously registered for that
command (spec §2.3, §3.1 — a resolved command runs a handler).

This is link 4 of §2.6's routing chain: the handler runs when the command
reaches this window surface unclaimed — no handler at the focused node
([`on_command_at`](Self::on_command_at)), none on an enclosing focus scope
([`on_scope_command`](Self::on_scope_command)), none on any ancestor. It
is the right level for a command that means the same thing anywhere in
this window, which is most of them; only a command whose meaning *depends
on where focus is* needs a positional registration.

The `command` symbol is app-owned and must be the **same** symbol the
application binds its `on-click` event to in the View (via
[`ViewEventBinding`](obscur_view::view::ViewEventBinding)); the runtime
only compares symbols, so no interner is shared. The handler typically
captures mutable app state through an `Rc<RefCell<_>>`/`Rc<Cell<_>>`.
Registering does not itself schedule a frame.

# Two accepted shapes (spec §2.3)

`handler` may be either

```ignore
rt.on_command(SAVE, || save());                        // takes nothing
rt.on_command(PICK, |e: &mut UiEvent| pick(e));        // takes the event
```

because [`IntoCommandHandler`] converts both into one stored shape. The
event form is how §2.3's API becomes reachable: `e.target()` is the node
that was hit, `e.current_target()` the node whose binding resolved,
`e.pointer()` / `e.key()` the §2.4 payload, `e.is_trusted()` the
trusted/untrusted origin, and `e.stop_propagation()` /
`e.prevent_default()` the steering. **All of it was unreachable before**:
this method used to take `impl FnMut()`, a closure given nothing, so a
handler could not learn which of two nodes bound to the same command had
been clicked — and the only escapes were the bespoke per-case signatures
(`on_row_click`, `on_scroll`) carved out one interaction at a time.

The zero-argument form is not deprecated and is still the right shape for
most handlers; a one-argument closure needs its parameter annotated
(`|e: &mut UiEvent|`) because inference cannot pick the marker and the
closure signature simultaneously.
