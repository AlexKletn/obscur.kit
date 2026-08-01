A multi-layer `background-image` list: two or more comma-separated image
layers (see [`BackgroundLayer`]), in CSS author order (first = top). CSS
paints layers back-to-front, so the scene coordinator emits the list in
reverse (last-listed = bottom, painted first). The **one-layer** case is
never this variant — it keeps the flat single-value variant
([`LinearGradient`](Self::LinearGradient) / [`RadialGradient`](Self::RadialGradient)
/ [`ConicGradient`](Self::ConicGradient) / [`BackgroundImageUrl`](Self::BackgroundImageUrl))
so a single-layer background stays byte-identical (spec §5.2).
