Creates a device-requiring custom pass with the given `name`.

A custom pass records GPU commands, so it defaults to requiring a device;
use [`Self::device_free`] for a pass that does not.
