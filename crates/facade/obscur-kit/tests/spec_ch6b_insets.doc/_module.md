§6.15 Insets and virtual keyboard — CSS environment values.

Specification-derived. The expected numbers below come from the case's host
setup (a notched phone reporting a 44px top and a 34px bottom safe-area
inset), never from any constant in this repository.

NOTE ON THE SETUP. §6.15 requires "Typed environment values: - safe areas"
*and* "CSS environment values may expose safe-area insets", so a case
asserting the second must perform the first. When these cases were written
the crate exposed no API by which a host could supply safe-area insets, so
the setup half could not be written and only the CSS half was asserted —
against a host that reported nothing. `UiRuntime::set_safe_area_insets` now
exists, so the host half is written here, exactly as the "Host:" line below
always described it. **No expected value changed**: the 44 and the 34 are the
ones the case was derived with.
