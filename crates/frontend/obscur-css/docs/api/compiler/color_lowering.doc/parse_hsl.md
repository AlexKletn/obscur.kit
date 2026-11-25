Parses `hsl()`/`hsla()` into a [`StyleColor`] (spec CSS Color 4 §7).

Accepts both the legacy comma syntax (`hsl(120, 100%, 50%)`) and the modern
space syntax with an optional `/ <alpha>` (`hsl(120 100% 50% / 0.5)`);
`hsla` is an alias. Saturation/lightness are `<percentage>` or `<number>`
(interpreted as the percentage value); hue is an angle. HSL is defined on
gamma-encoded sRGB, so no transfer function is applied.
