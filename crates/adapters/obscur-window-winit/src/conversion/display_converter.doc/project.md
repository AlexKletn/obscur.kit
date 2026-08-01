Projects a monitor `snapshot` onto a domain [`Display`] with the
caller-supplied `id` (spec §6.6).

The size and position casts to `f32` are lossless enough at UI scale:
monitor pixel dimensions and coordinates stay within the precise integer
range of `f32`. The refresh rate is preserved exactly in millihertz.

[`Display::available_modes`], [`Display::work_area`], and
[`Display::color`] are left at their empty defaults: winit 0.30 exposes
no reserved-region and no color/HDR query at all, and its
`video_modes()` enumerates *exclusive-fullscreen* modes, which this
backend refuses to present (`supports(ExclusiveFullscreen)` is
`Limited`, degraded to borderless), so nothing downstream could act on
the list.
