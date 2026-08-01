Emits a `<For>` as a real Rust `for` loop, with its `empty` / `separator`
slots (spec §2.11).

The shape:

```ignore
let mut __first = true;            // only when a slot needs it
for item in &items {
    if !__first { /* separator */ }
    __first = false;
    __nodes.push(/* body, keyed */);
}
if __first { /* empty */ }
```

One flag serves both slots, and that is not a trick: "no separator before the
first item" and "the collection was empty" are the same question — did an
iteration happen yet.

`as={item}` names the loop variable, and it is a Rust binding like any other:
the body's `{item.name}` is then an ordinary path (see the module docs).
Without `as` the item is bound to `_`, which renders the body once per item
with no way to read it — a repeat, which is what a `<For>` that names nothing
asks for.

The collection is walked with `.iter()`, not `&{each}`. Both borrow, so a
`<For>` never consumes the caller's data — but `&{each}` breaks the moment the
binding is already a reference (`&[T]` becomes `&&[T]`, which is not an
iterator), and a caller passing a slice is not doing anything unusual.
`.iter()` reads the same on a `Vec`, a slice, an array and a borrow of any of
them.
