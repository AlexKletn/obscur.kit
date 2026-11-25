The current power state that feeds adaptive quality (spec §7.11).

[`PowerProfile`] is the pure snapshot of power conditions — mains vs battery,
charge level, OS low-power mode, and thermal pressure — that the runtime reads
to decide whether to lower rendering quality (spec §7.11). Its
[`is_constrained`](PowerProfile::is_constrained) predicate is the single
signal the runtime acts on. The colocated [`PowerSource`] and [`ThermalState`]
describe where power comes from and how much thermal headroom remains.
