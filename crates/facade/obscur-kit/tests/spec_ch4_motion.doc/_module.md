**SPECIFICATION-DERIVED** transition and animation cases for obscur.kit §4.17
("CSS `transition`, `@keyframes` and `animation-*` are parsed as a
compatibility/declarative frontend and compiled into native timelines";
"time-driven motion"; "interruption; reversal"; "style interpolation").

Every expected number below was produced by an agent reading the specification
— obscur.kit §4 and the W3C css-transitions-1 / css-animations-1 /
css-cascade-5 / css-color-4 text it normatively cites — with **no access to
this repository's source**. A failure here is a defect report about the
engine, and must not be "corrected" by editing the expected value.

Translation note: the deriver triggers each transition by "adding class `on`".
Here the trigger is `:hover` via [`UiRuntime::set_pseudo_state`], which is the
style-change mechanism this workspace exposes. Nothing in any case depends on
*which* style change fires the transition.
