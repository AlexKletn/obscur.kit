The [`SemanticValue`] a control publishes, or [`SemanticValue::None`] for a
role that has no value of its own.

Only roles that genuinely carry a position answer here. Telling a screen
reader "this is a slider" and then nothing about where it is, is worse than
not exposing the control at all — the user is told a thing exists and given
no way to know its state.
