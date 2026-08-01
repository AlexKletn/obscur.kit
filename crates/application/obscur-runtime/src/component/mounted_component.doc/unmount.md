Runs the component's `on_unmount`, then retires the instance (clearing its
subscription scope).

Called *before* the value is dropped, so a component can act while its
state is still readable — releasing the state itself needs no hook.
