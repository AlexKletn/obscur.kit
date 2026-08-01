The §2.10 check with the set of `.okui`-declared **explicitly exported pure
functions** in scope, so a call to one of them is admitted where an
unconstrained free call is refused (spec §2.10).

[`check_expression`] is this with an empty `functions` — every free call
refused — which is what the macro front end wants and what the rule did before
the exported-pure exception.

# Errors

As [`check_expression`].
