The names of `.okui`-declared **explicitly exported pure functions**, so a
call to one is admitted where an unconstrained free call is refused
(spec §2.10). Empty by default — [`new`](Self::new) — which is what the
macro front end wants, since it has no way to declare a pure function.
