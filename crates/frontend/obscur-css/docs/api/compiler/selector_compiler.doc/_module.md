Computing selector specificity and minting selector ids (§4.13/§4.14).

[`SelectorCompiler`] turns a parsed selector into a [`CompiledSelector`]: the
CSS `(a, b, c)` specificity triple the cascade ranks declarations by, plus a
sequential [`SelectorId`] handle that ties a produced [`Declaration`] back to
the rule that authored it. It also surfaces the excluded dynamic-state
pseudo-classes (`:link`, `:visited`, `:target`) and pseudo-elements
(`::before`, `::after`, …) as [`UnsupportedCssFeature`]s (§4.13) rather than
silently matching them.

[`Declaration`]: obscur_style::cascade::Declaration
