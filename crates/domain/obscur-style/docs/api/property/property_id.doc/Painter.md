Binds a registered canvas painter to the element, by name.

The element's box becomes a drawing region and the named painter fills it, which
is how a chart, a dial or a waveform reaches CSS at all — nothing in any CSS
specification names a drawing routine.

It is `-obscur-` prefixed for exactly the reason browsers prefix their own
extensions: a stylesheet carrying it stays valid CSS everywhere, and every
other renderer ignores it.

`none` is the initial value and unbinds any painter a lower origin set.
