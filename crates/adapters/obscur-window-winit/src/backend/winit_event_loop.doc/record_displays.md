Replaces the display topology with a fresh enumeration, reporting what
changed as [`DisplayEvent`]s (spec §6.6 "hotplug/configuration events").

winit 0.30 delivers no monitor notification of any kind, so the events
§6.6 requires can only be *derived*, by diffing one enumeration against
the last. That diff is the decision in this subsystem, and it lives here
rather than in the handler for the same reason focus tracking does: the
handler's callbacks only run inside a real winit loop.
