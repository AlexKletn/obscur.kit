Parses `rgb()`/`rgba()` arguments into a [`StyleColor`].

Colour channels accept numbers (`0..=255`) or percentages (`0%..=100%`);
alpha, when present, is a `0..=1` number or a percentage. Commas and a `/`
separator are skipped. Requires at least three channels.
