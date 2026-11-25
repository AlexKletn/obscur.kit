[`UiRuntime`]'s observability surface: the two sinks a host installs (§1.6
diagnostics and §7.9 telemetry), the per-frame budget the telemetry checks
against, and the §7.6 inspection surface — the mode toggle, the hovered node's
info, and the whole-tree snapshot an agent asserts on.

They are one subject because they share one property: an unconfigured runtime
pays nothing for having them. A sink that is never installed makes every emit a
no-op, and the two inspection readers derive their answers from the retained
tree and the boxes the frame already produced rather than from anything kept for
them.

`set_inspect_mode` is the one exception, and it is the exception that states the
rule: enabling the overlay is the only thing in the file that changes what a
frame paints, which is why it is also the only thing here that asks for one.
