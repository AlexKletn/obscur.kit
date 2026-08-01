`font-family: monospace` must reach the mono face, not merely the first
registered one — the two are registered under distinct generics, and a resolver
that ignored the family would hand back Noto Sans for both.
