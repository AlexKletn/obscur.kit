//! Measure/arrange layout contracts and the layout graph (skeleton).
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod model;

pub use model::LayoutBox;

pub mod algorithm;
pub mod context;
pub mod invalidation;
pub mod observer;
pub mod pass;
// There is no `port` module. It held `IntrinsicMeasure` — the port whose docs
// claimed leaves routed to it — with `LayoutEnvironment` as its argument and
// `MockIntrinsicMeasure` as its only implementation, and nothing outside the
// module ever named any of the three. It was deleted rather than wired, because
// both leaf kinds already have a road and neither has this port's shape: a text
// leaf is intercepted before an algorithm is chosen and measured by
// `measure_text_node` off seventeen inputs (families, OpenType attributes,
// bidi direction, the `ParagraphShaper`, the hyphenator, the live editing
// buffer) that `LayoutEnvironment`'s scale-factor-and-viewport cannot carry;
// a replaced element is sized BEFORE layout runs, by `size_replaced_elements`
// rewriting `auto`/`auto` to the decoded image's logical `Px`, so no
// measurement happens at all. Wiring it would also have been a second spelling
// of `ChildMeasurer`: only the runtime's `RecMeasurer` holds enough state to
// implement it, its `measure` would delegate to the function `measure_child`
// already delegates to, and no algorithm could call it anyway — algorithms
// address children by index into `&[LayoutChild]`, the port by `NodeId`.
// `IntrinsicSize` went with it: grid's `Contributions { auto, min, max }` is
// the min/max-content record production actually fills.
pub mod virtualization;
