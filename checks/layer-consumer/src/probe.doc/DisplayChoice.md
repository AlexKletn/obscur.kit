The three answers an application actually has to handle.

Written as an enum rather than an `Option` because "this desktop has no layer
surfaces" and "no monitor by that name" call for different behaviour: the first
means open a window instead, the second means carry on wherever the platform
puts it.
