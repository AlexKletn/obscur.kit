Takes `event`'s pointer's outstanding press and resolves what the release
amounts to against the node the event landed on.

The press is consumed either way: the anchor is cleared on every release,
which is what keeps `:active` from sticking on a control the pointer has
let go of.
