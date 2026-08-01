Rasterizes a CSS `box-shadow` onto a [`vello::Scene`] (spec §5.2,
css-backgrounds-3 §6.1).

A shadow is not a path the scene model carries: it is a *blurred* rounded
rectangle derived from the box's border box, its `spread`, its `offset` and its
corner radii, and the inset form is a soft frame punched out of the box
interior. That derivation — the §6.1.1 radius easing, the CSS-radius →
gaussian-σ mapping, and the `DestOut` punch — is this file's whole vocabulary;
[`draw_shadow`] is the single entry point the command walk calls.
