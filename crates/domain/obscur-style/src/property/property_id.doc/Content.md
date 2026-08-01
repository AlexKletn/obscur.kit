The `content` property (a [`PropertyValue::String`](crate::property::PropertyValue::String)
literal; NOT inherited). Only meaningful on the `::before` / `::after`
generated-content pseudo-elements (§4.16): it never computes onto a real
element's box (it is absent from the resolver's computed-property set), and
the runtime reads it off a matched pseudo-element rule to paint a
decoration-only inline box. Only the plain string form is modelled; `none`
/ counters / `attr()` are out of scope.
