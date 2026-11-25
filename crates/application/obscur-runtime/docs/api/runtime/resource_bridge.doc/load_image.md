Runs one image load — off the frame thread when a blocking executor is
installed, deferred to after the frame phase (frame thread, on poll) when
none is (spec §3.3).

Both paths end in the same place: `apply_outcome`, which lets the hub rule
on staleness. The only difference is *when*, and neither reaches the loader
inside `render_now`: with an executor the load leaves the frame thread; with
none it is queued and run after the frame phase (§3.4, §5.15).

The DPI-variant ladder (spec §5.16) is built **here**, on the frame thread,
because `device_scale` is known there; the load job then only walks a list of
candidate paths and needs no access back to the runtime.
