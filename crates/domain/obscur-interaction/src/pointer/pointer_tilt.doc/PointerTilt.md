§2.4's pointer **tilt**: the angle between the pointer's axis and the surface
normal, in degrees, resolved per plane.

§2.4 lists tilt as a required pointer fact, alongside pressure and rotation,
because a pen is not a point: a drawing surface needs to know which way the
barrel leans to shape a stroke, and only the device can say.

Both angles are in `-90..=90` degrees, matching the Pointer Events convention:
[`x_degrees`](Self::x_degrees) is the tilt in the plane containing the Y axis
(a lean left/right, negative toward the left), and
[`y_degrees`](Self::y_degrees) is the tilt in the plane containing the X axis
(a lean toward/away from the user, negative toward the top). A device held
perpendicular to the surface, and every device that cannot report tilt at all,
reports [`UPRIGHT`](Self::UPRIGHT) — zero on both axes.

A mouse or a touchscreen contact therefore reports `UPRIGHT` too. That is a
deliberate flattening rather than a missing value: §2.4 requires the field on
every pointer event, and "no lean" is the honest reading of a device with no
barrel.
