Owns the surface and does the one blocking thing, on a thread of its own.

`wgpu::Surface::get_current_texture` waits for the compositor to release a
buffer. On a vsync-paced surface that is a frame; under load it was measured at
**544 ms**. Whatever thread makes that call is doing nothing else for its
duration — and in a single-threaded UI loop that thread is also the one reading
input, so touches queue in the kernel and arrive in a burst afterwards.

The spec forbids this in three places: platform calls are to be asynchronous and
synchronous work "short, deterministic and frame-bounded" (core principle 5),
*submit/present* is step 12 of the frame pipeline whose synchronous phase "must
stay short and deterministic" (§3.4), and hidden synchronous blocking is grounds
for redesign (§9). What is **not** forbidden is vsync itself — §3.5 endorses it.
The two are compatible exactly when the wait happens off the input path, which
is what this type is.

It is a **pull**, not a stream: nothing is acquired until
[`request`](Self::request) asks, so an idle loop does not sit holding a
swapchain image the compositor could be using. When one lands the wake callback
fires, the loop's own wait ends, and [`try_take`](Self::try_take) has it ready.

A loop that finds no image simply does not draw this turn. §3.5 permits that in
so many words — "stale intermediate frames may be skipped" — and it is the right
answer: the alternative is to wait, which is the thing being removed.
