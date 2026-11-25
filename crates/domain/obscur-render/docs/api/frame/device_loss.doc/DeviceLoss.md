A description of a GPU device-loss event and whether it is recoverable
(spec §5.2).

Backends must handle device loss explicitly (spec §5.2). This value records
the [`DeviceLossReason`], whether the runtime may rebuild the device and
continue, and an optional human-readable detail.
