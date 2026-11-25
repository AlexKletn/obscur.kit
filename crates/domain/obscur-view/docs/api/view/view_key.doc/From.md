The owned twin of the [`&str`](str) conversion.

It exists so a caller holding a `String` key needs no `.as_str()` — which
matters most to generated code: `.okui`'s `key={item.id}` emits one expression
whatever the field's type is, and `u64` and `String` keys have to travel the
same path or the emitter would need to know the type it is about to hand to
rustc.
