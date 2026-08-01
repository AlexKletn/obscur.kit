Resolves one margin longhand into a [`Margin`]: an authored `auto` stays
[`Margin::Auto`] and an authored `%` stays [`Margin::Percent`], while every
other value folds to logical pixels exactly as [`length_px_of`] does.
