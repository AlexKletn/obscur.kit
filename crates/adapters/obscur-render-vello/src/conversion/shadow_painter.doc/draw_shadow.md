Rasterizes a CSS `box-shadow` as a gaussian-blurred rounded rectangle
(spec §5.2).

The box's border box is translated by `offset` and inflated (a drop shadow) or
deflated (an inset shadow) by `spread`; Vello's
[`Scene::draw_blurred_rounded_rect`] then blurs it, mapping the CSS blur
*radius* to a gaussian standard deviation of `blur / 2`. Vello takes a single
corner radius, so the four `radii` collapse to their maximum (a documented
approximation for non-uniform corners). An **inset** shadow is approximated by
the same blurred rounded rect at the deflated geometry — Vello does not clip it
to the box interior here, so inset fidelity is approximate (documented). A
degenerate (non-positive) rect is skipped.
