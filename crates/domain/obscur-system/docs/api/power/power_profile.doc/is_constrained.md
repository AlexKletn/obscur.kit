Returns whether the runtime should lower quality under this profile
(spec §7.11).

The profile is constrained when any of these hold: OS low-power mode is
engaged; the device is on battery below
[`LOW_BATTERY_THRESHOLD`](Self::LOW_BATTERY_THRESHOLD) (20%); or thermal
pressure is [`ThermalState::Serious`] or worse.
