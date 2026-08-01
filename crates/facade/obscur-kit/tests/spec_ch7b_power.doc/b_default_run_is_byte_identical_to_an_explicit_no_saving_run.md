§7.11, verbatim: "It is off by default and must not compromise critical
responsiveness."

"Off by default" is only meaningful as an equivalence: the behaviour of a run
with no power configuration must equal the behaviour of a run that
explicitly asks for no power saving. Run D constructs the runtime with
defaults only; run F constructs the `PowerProfile` denoting full performance
and supplies it. Both drive the same effect-bearing app — shadowed card,
blurred backdrop, long list, running transition — for a fixed number of
frames against a deterministic clock.
