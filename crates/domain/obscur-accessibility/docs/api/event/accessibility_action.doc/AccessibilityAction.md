An inbound action request delivered from assistive technology (spec §7.1).

When a screen reader or other AT asks to activate, focus, or set the value of
a node, a platform bridge (UIA / AX / AT-SPI / web) delivers it up as one of
these. It then enters the **same command/event model as ordinary input**: an
AT-requested activation is dispatched exactly like a pointer or key press.

The [`value`](Self::value) payload is carried only for value-setting actions
such as [`SemanticAction::SetValue`]; it is `None` otherwise.
