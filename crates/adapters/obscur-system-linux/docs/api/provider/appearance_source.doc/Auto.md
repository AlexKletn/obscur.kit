Use the portal and the settings provider for the active desktop session.

The portal takes precedence field by field. GNOME fills missing fields from
GSettings, while KDE Plasma fills them from `kdeglobals` and `kwinrc`.
When the session is unknown or unsupported, only the portal is consulted.
