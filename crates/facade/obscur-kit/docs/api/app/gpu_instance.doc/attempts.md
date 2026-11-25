The backend sets `open` tries, in order, and the reason this is a function with
no arguments.

1. `Backends::PRIMARY` — Vulkan, Metal, DX12, WebGPU. The one that succeeds
   everywhere this program is actually deployed, and the reason the process
   stops bringing up a GLES stack it never draws with; `_module.md` weighs it.
2. `Backends::all()` — everything compiled in, GLES included. Reached only when
   the first attempt created no surface or found no adapter, so a machine with
   no usable Vulkan behaves exactly as a single all-backends instance did, one
   failed `request_adapter` later.

The two sets are fixed, non-empty, and different, and the last one is
everything. That is the whole invariant, and it is what
`the_last_attempt_is_everything_compiled_in` holds.

# Why `WGPU_BACKEND` is not read here

It was, for one revision of this file, as `Backends::default().with_env()`, and
that was a defect this file introduced rather than inherited: before this module
existed both runners said `wgpu::Instance::default()`, which is
`InstanceDescriptor::new_without_display_handle()` — `Backends::all()`, no
environment anywhere in it.

`Backends::with_env` **replaces** the set, it does not filter it. So the
narrowing was unbounded from below:

* `WGPU_BACKEND=dx12` on Linux — a set with no HAL compiled in;
* `WGPU_BACKEND=d3d11`, or any typo — `from_comma_list` warns and returns
  `Backends::empty()`.

In both cases every attempt in the plan was that same empty set, the instance
came up with nothing in it, and the process died before its first frame with
`Failed to create surface for any enabled backend: {}`. That is a way to have no
keyboard at all, reachable from one exported variable, in exchange for a knob
nobody had asked for.

The knob is not worth re-adding in a filtering form either. A set intersected
with `PRIMARY | GL` can still be empty, so a filter needs the same fallback that
would then make the knob non-binding — a forcing switch that does not force is
worse than none, because it answers a debugging question with a lie. Whoever
needs one backend and no other should build with the other's cargo feature off,
which the answer cannot silently work around.

`InstanceFlags` is still read from the environment, and the difference is the
blast radius: those bits can make a run slower or noisier, never absent.
