The chord held when the key transitioned.

The runtime is authoritative here and stamps this field before the event is
delivered, folding the state from the modifier transitions it has seen and from
any correction a platform adapter pushed through `UiRuntime::set_modifiers`. An
adapter may pre-fill it; the runtime overwrites it, because two sources of
truth for one question is how a chord goes stale after a focus loss.

It exists on the event rather than only on the runtime so a handler holding the
event can answer "was Ctrl down?" without asking for ambient state that may
have changed since — which is also what makes a key event queueable and
replayable.
