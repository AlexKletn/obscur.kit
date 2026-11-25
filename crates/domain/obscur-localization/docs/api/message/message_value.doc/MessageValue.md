One value bound to a message parameter.

The variants are the three things a message can *do* with an argument, and
they are distinct because the operations are distinct: text is interpolated,
a number is interpolated **and** can be pluralised (which needs its
[`PluralOperands`], not its value), and a selector is only ever matched
against arm names.
