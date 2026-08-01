[`UiRuntime`]'s pointer surface: taking and releasing a capture, and the §6.9
drag-and-drop API — beginning a drag, the drop targets a node registers, the
live session, the commit or discard a release resolves to, and the outcome a
host drains afterwards.

Capture and drag are one subject rather than two: a drag is what a captured
pointer's sequence becomes once it crosses the slop threshold, and the same
teardown ends both. What is *not* here is the routing — which event reaches a
captured node, and which move tracks an in-flight drag across drop targets — as
that is the pointer dispatch's ordering problem and stays with it in
[`pointer_dispatch`](super::pointer_dispatch). This file is the surface a host
and a control call, plus the one step of the drag's lifecycle that only the
runtime can take: committing the drop's staged work through the same enactor a
reconcile is validated by.
