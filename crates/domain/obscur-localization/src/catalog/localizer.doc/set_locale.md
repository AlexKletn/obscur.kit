Switches the current locale, recomputing its plural rules.

The rules are derived here and nowhere else, so a locale switch cannot
leave the previous language's categories behind — the failure mode that
would show as Russian text pluralised by English rules.
