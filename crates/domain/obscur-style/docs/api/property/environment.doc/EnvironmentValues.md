The environment values in force, in logical pixels (spec §6.15).

All zero by default, and that default is **correct rather than merely safe**:
a display with no cutout and no on-screen keyboard genuinely has no insets, so
a desktop host that reports nothing gets the right answer without knowing this
type exists. A host that does know — a tablet shell, a mobile platform — sets
what it measures, and the values change over the app's life as the keyboard
comes and goes or the device is rotated.
