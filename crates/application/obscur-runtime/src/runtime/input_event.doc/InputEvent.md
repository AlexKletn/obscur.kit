A platform input event, queued to be dispatched by the frame pipeline rather
than the instant a host hands it over (spec §3.4 phases 1 and 3).

# Why the four kinds are one type

A host receives pointer, keyboard, IME and scroll events from one place, in one
order, and that order is part of what they mean: a key that arrives between two
clicks was pressed between them. Four separate queues could not say that, and
four separate immediate calls only say it by accident of when the host happened
to call them. Sharing one queue makes the ordering the runtime's to keep rather
than the host's to be careful about.
