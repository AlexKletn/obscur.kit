Maps a winit [`KeyCode`](winit::keyboard::KeyCode) to the domain's
layout-independent [`PhysicalKey`], or `None` for a position the domain
does not model (spec §6.7).

The winit `SuperLeft`/`SuperRight` positions map to the domain's
`MetaLeft`/`MetaRight` (the `Meta` / `Super` / `Command` modifier).
