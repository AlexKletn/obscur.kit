Returns a full structured snapshot of the live tree — one
[`NodeInspection`](crate::orchestration::NodeInspection) per node in pre-order — for an agent to observe and
assert on in a headless run (spec §7.6 inspectable tree/styles/layout).

Each node carries its identity (kind, component, `type#id.class`), its
literal text (for text nodes), its absolute border box (when laid out this
frame), and a few key computed styles. This is the read half of the
agent debugging loop; the write half is
[`handle_pointer`](Self::handle_pointer)/[`handle_key`](Self::handle_key).
