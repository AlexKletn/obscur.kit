How much of the keyboard a layer surface asks for.

The default is [`Never`](LayerFocusPolicy::Never) and that is the right
default: a panel that silently took keyboard focus would swallow every
shortcut on the desktop.

The three are ordered by escalation, and only the first two are universally
expressible. [`Exclusive`](LayerFocusPolicy::Exclusive) asks to receive keys
that would otherwise go to the focused window, which some platforms offer
outright and others do not offer at all; a backend that cannot answers
[`LayerFeature::ExclusiveKeyboard`](crate::port::LayerFeature::ExclusiveKeyboard)
with what it can actually do instead of pretending.
