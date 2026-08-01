§11.5 sub-step (d) — grow the *growth limit* of the tracks spanned by one item
whose **max** sizing function is `kind`, to fit that kind's contribution.
Space goes only to tracks whose limit is still infinite (or newly
`infinitely_growable`); if none, it is distributed beyond limits to all
affected tracks.
