Returns the declared named slots (spec §2.12); an empty slice means the
component declares none, which *closes* the slot surface rather than
leaving it open — the asymmetry with props/children is deliberate, since
a `slot="…"` naming nothing has no meaning to fall back to.
