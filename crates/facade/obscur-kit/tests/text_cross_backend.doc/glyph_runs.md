How many glyph runs the Vello converter encoded for these commands.

The GPU-free half of the guard. A run is one path and one draw object, so the
run count *is* the composite count: it answers "did the converter fold these
commands together?" on any machine, CI included, without rendering anything.
