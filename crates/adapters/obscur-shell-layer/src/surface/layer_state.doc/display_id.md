The id for the output at this position in the compositor's enumeration.

Positional because Wayland gives an output no stable identifier of its own that
survives across compositors — the connector name is the closest thing and it is
a string, which is exactly what the settings stopped carrying. The id is stable
for the life of the connection, which is the life of the surface, and that is
the span over which anything asks.
