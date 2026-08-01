Builds a `View` from a TSX-like element tree, lowered to ordinary Rust at
compile time.

```ignore
let view = okui! {
    <container class="row">
        <Button label="OK" on:click={on_ok}/>
        "Ready"
    </container>
};
```

An UPPERCASE-initial tag (`Button`) is a component and resolves to its real
`ComponentId`; a lowercase tag (`container`) is an element. See the `dsl`
module's docs for the full grammar.

The expansion is an expression and needs no `use` items — every emitted path
is absolute, and written through the `obscur-kit` facade
(`::obscur_kit::view::view::…`, `::obscur_kit::identity::…`). So the calling
crate needs exactly the one dependency an application already has, and no
internal crate: that is [`CodegenRoot::facade`], and the emitted paths used
to be `::obscur_view` / `::obscur_identity` — which resolve only inside this
workspace, because a `::`-prefixed path is looked up in the crate's *direct*
dependencies.

Malformed input (an unclosed or mismatched tag, a non-expression `{...}`) is
reported as a compile error at the offending span.
