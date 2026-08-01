Returns the natural size for `index`, clamped into `constraints`.

Out-of-range indices measure as [`Size::ZERO`] (still clamped into
`constraints`, so a non-zero minimum is honoured).
