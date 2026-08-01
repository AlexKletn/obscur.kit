Records the constructs the kit recognises but cannot honour, found anywhere
in the complex selector (§4.13): the excluded dynamic-state pseudo-classes
(`:link`, `:visited`, `:target`) and every pseudo-element.

Both lower to something the matcher never matches, so the rule is inert; the
diagnostic is what keeps that from being a silent drop.
