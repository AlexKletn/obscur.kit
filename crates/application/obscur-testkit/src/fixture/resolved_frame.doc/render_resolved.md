Renders, then drives the deferred no-executor load to completion, then renders
again so the decoded image sizes and composes — returning the second frame.

With no blocking executor installed a load is queued out of the frame phase and
runs on the frame thread when `poll_resource_tasks` is called, *after* the frame
(spec §3.3/§3.4). A single `render_now` therefore no longer resolves an image
in-frame; this is the "render → drain → render" a real host performs across its
event loop, collapsed for a test. It is the snapshot normalization §2.4 asks
this crate for: without it every image assertion would open with the same three
lines and one of them would eventually be forgotten.
