Trusted custom render-pass extensions for the Vello backend (spec §5.3).

Spec §5.3 permits trusted extensions to inject custom render passes behind
capability checks. This module holds the adapter-side descriptor for such a
pass; the actual wgpu command recording is a future refinement.

- [`VelloCustomPass`] — a named custom pass and its device-permission rule.
