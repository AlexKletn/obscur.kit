The node that currently holds focus, if any.

Tracked directly rather than via
[`FocusTree`](obscur_interaction::focus::FocusTree): the focus tree
requires nodes to be pre-registered as [`FocusNode`]s, which this runtime
does not yet do, so a plain `Option<NodeId>` is the simpler correct model
for pointer-press focus (documented fallback).
