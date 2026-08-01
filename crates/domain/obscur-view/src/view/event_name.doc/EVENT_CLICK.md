Reserved symbol naming the primary **click** event — a press-then-release on
the same element (spec §2.3, §3.1). The deliverable event name: a
[`ViewEventBinding::new(EVENT_CLICK, command)`](super::ViewEventBinding::new)
on an element fires `command` when the runtime detects a click on it.
