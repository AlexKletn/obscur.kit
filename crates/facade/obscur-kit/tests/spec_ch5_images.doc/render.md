Renders `css` over the picture view in a runtime sandboxed to `dir`.

With no blocking executor the image load is deferred out of the frame phase
(§3.4/§5.15), so this drives it to completion — render (which queues the
load), `poll_resource_tasks` (which runs it on the frame thread, after the
frame), then render again so the now-decoded image sizes and composes.
