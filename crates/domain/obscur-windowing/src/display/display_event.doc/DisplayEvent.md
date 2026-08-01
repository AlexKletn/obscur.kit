A hotplug or configuration change reported for a display (spec §6.6).

These notifications track the monitor list and its configuration over time:
displays being connected or disconnected, a display's mode/scale/bounds
changing, and the primary display changing. Every variant carries the
[`DisplayId`] it concerns.
