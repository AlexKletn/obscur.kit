The bare identifier a call's callee names, if it is a single-segment path
(`fmt_price`), else `None` (`std::fs::read`, `make()()`, a method call).

Only a bare name can match a `.okui`-declared pure function, and only a bare
name can be reported by name; a qualified or computed callee is refused with
the generic [`FORBIDDEN_FREE_CALL`] message.
