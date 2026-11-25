The priority band a piece of frame work belongs to (spec §3.6).

The variants are declared in priority order, highest urgency first, so the
derived [`Ord`] makes the *most urgent* band the *smallest* value. A
min-first queue (see [`SchedulerQueue`](super::SchedulerQueue)) therefore
pops the most urgent work. In short: **lower [`Ord`] == higher urgency**.

The bands map onto the numbered priority list in spec §3.6:

1. input / focus / pointer / IME → [`SchedulerPriority::Input`]
2. UI transaction commit → [`SchedulerPriority::Commit`]
3. current-frame animation, layout and paint → [`SchedulerPriority::CurrentFrame`]
4. resource completion → [`SchedulerPriority::ResourceCompletion`]
5. background shaping / decoding / cache → [`SchedulerPriority::Background`]
6. telemetry and development diagnostics → [`SchedulerPriority::Diagnostics`]
