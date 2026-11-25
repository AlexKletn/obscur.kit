[`UiRuntime`]'s event surface (spec §6.5): the capture/bubble listeners a node
carries, and the two node-authored events that are not commands — `input` and
`change`.

It is separate from `command_routing.rs` because the vocabulary is: a command is
a named action that travels a routing chain and can be enabled, disabled and
bound to a shortcut, whereas a listener is notified that something happened to a
node and cannot refuse it. Both are installed on the same registry; that is an
implementation fact, not a reason to read them together.
