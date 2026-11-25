Reports how faithfully this backend can satisfy `mode` (spec §6.5, §6.2
result categories).

Borderless fullscreen is winit's one portable form, so it is the only one
reported `native`. Exclusive and presentation forms are silently *served* by
borderless and therefore report `conditional`, agreeing with
supports(ExclusiveFullscreen). Kiosk reports `Unsupported`, agreeing with
supports(Kiosk): a borderless window the user can escape is not a kiosk.

The optional §6.5 controls are the other half. Hiding the cursor, inhibiting
sleep, choosing the target monitor, and any non-`Allow` safe-escape policy are
all unwired here, so requesting one downgrades an otherwise native outcome to
`conditional`. [`PresentationMode::restore_previous_state`] is *not* checked:
leaving presentation resets the window to `Normal`, which restores the default
case but loses a pre-presentation `Maximized` state — a known gap this
function does not currently report.
