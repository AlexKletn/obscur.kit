How many IR nodes the data can change — the size of the per-frame walk that
is *not* a plain [`UiIrViewBridge`] call. `0` means this screen is entirely
static and its lowering is the bridge's, node for node.
