How many times [`ComponentRenderPass::run`] re-renders newly appeared
components within one frame before giving up.

A component's output can contain further component elements, so rendering is a
fixpoint. This bound is not a design limit on nesting depth — real trees nest
far shallower — but a backstop: a component that renders itself, directly or
mutually, would otherwise spin the frame forever. Exceeding it truncates that
frame's tree instead of hanging, and the `component.rendered` counter shows it.
