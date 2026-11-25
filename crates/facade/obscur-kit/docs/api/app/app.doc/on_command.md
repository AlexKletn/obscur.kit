Registers `handler` for `command` at the **window** level — link 4 of §2.6's
routing chain, and the right level for a command that means the same thing
anywhere in this window.

# Two accepted shapes (spec §2.3)

```ignore
.on_command(SAVE, || save())                          // takes nothing
.on_command(PICK, |e: &mut UiEvent| pick(e.target()))  // takes the event
```

The event form is how §2.3's model reaches an application: `e.target()` is the
node that was hit, `e.current_target()` where the binding resolved,
`e.pointer()` / `e.key()` §2.4's payload, `e.trust()` the trusted/untrusted
origin, and `e.stop_propagation()` / `e.prevent_default()` the steering. Until
this widened, the handler was an `FnMut()` given nothing, so none of it was
reachable and a handler had to *capture* everything it might need.

A one-argument closure needs its parameter annotated (`|e: &mut UiEvent|`) —
inference cannot pick the marker and the closure's signature at once. The
zero-argument form is unchanged and is still right for most handlers.

For a handler that must mean different things in different places, register it
at the node instead: [`on_command_at`](Self::on_command_at).
