Renders and presents `submission` to `target` (spec §5.2).

On success returns a [`PresentResult`] echoing which scene generation
reached the screen. An effect the backend cannot reproduce yields a
documented fallback `Ok` — with the diagnostic of point 2 above raised, so
the fallback is never silent — or, absent any fallback,
`Err(`[`RendererError::UnsupportedEffect`]`)`. Never a panic. Device loss
surfaces as `Err(`[`RendererError::DeviceLost`]`)` or
[`PresentResult::DeviceLost`].

# Errors

Returns a [`RendererError`] when the submission cannot be presented — for
example an unsupported effect or a lost device.
