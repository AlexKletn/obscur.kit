Converts either accepted handler shape into the one stored
[`CommandHandler`] (spec §2.3).

# What this trait is for

§2.3 requires that a handler be able to read `target` / `current_target`, the
device payload and the trust flag, and to call `stop_propagation`,
`stop_immediate_propagation` and `prevent_default`. All of that needs the
event, so the handler signature has to widen to `FnMut(&mut UiEvent)`. But the
zero-argument form is the *right* shape for the majority of handlers — an
increment, a navigation, a quit — and forcing every one of them to accept and
ignore a parameter would be a tax on the common case. So both are accepted:

```ignore
rt.on_command(SAVE, || save());                                  // WithoutEvent
rt.on_command(PICK, |e: &mut UiEvent| pick(e.target()));         // WithEvent
```

# Why a marker generic, and not one trait

A single trait cannot cover both signatures. The natural attempt —

```text
impl<F: FnMut()> IntoCommandHandler for F { … }
impl<F: FnMut(&mut UiEvent)> IntoCommandHandler for F { … }
```

— is rejected as a conflicting implementation: coherence compares the *self*
types, and both impls have the self type `F`, so the compiler must assume some
future type implements both `FnMut()` and `FnMut(&mut UiEvent)` (a hand-written
type trivially can) and refuses the pair. Adding the marker type parameter
makes them impls of two distinct traits, which is legal, while the call site
stays a single method: `on_command<M, H: IntoCommandHandler<M>>` lets inference
pick `M` from the closure's arity, because only one of the two candidate impls
has its `FnMut` bound satisfied.

# The one cost, stated plainly

Inference needs the closure's parameter type to be *known*, which for a
one-argument closure means annotating it: `|e: &mut UiEvent| …` rather than
`|e| …`. An unannotated `|e|` leaves both `M` and the closure's own signature
open at once and the compiler cannot resolve either from the other. The
zero-argument form needs no annotation.
