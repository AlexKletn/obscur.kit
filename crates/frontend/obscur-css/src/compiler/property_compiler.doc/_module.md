Lowering a single parsed declaration into an `obscur-style` value (§4.14).

[`PropertyCompiler`] maps one [`ParsedDeclaration`] onto exactly one
[`CompiledDeclaration`]:

* a name matching one of the modelled [`PropertyId`] longhands — the set
  [`super::longhand_catalog`] holds — becomes a cascade [`Declaration`]
  ([`CompiledDeclaration::Standard`]);
* a `--`-prefixed name becomes a [`CustomProperty`]
  ([`CompiledDeclaration::Custom`]);
* any other recognised-but-unsupported property name, or a value the crate
  cannot lower, becomes an [`UnsupportedCssFeature`]
  ([`CompiledDeclaration::Unsupported`]) rather than being dropped silently
  (§4.13).

What is left in this file is that contract and the dispatch that serves it:
[`lower_for_property`] asks which grammar a longhand is written in and hands
the value list to the module that owns it. The grammars themselves are the
entities beside it, because each is reachable from more than one property and
none of them is about *declarations*: the shared component vocabulary
([`super::value_lowering`]), the `<color>` syntaxes
([`super::color_lowering`]), the shorthand fan-outs
([`super::shorthand_expansion`], [`super::border_shorthand`]), and the
property families with a grammar of their own — grid tracks and lines, fonts,
shadows, filters, transforms, backgrounds and gradients. A value the crate
cannot lower degrades to [`CompiledDeclaration::Unsupported`] wherever it is
read, never to a silently dropped declaration.
