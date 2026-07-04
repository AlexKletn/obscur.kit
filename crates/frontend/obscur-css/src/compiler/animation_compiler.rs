//! Lowering CSS `animation-*` declarations into `@keyframes` references (§4.17).
//!
//! Unlike `transition`, an `animation` names one or more `@keyframes` rules to
//! play on the element plus a duration. [`AnimationCompiler`] extracts, from a
//! style rule's declarations, the list of [`AnimationRef`]s it declares: an
//! interned animation name (resolved against the stylesheet's compiled
//! `@keyframes` later, at [`compile`](crate::compiler::StylesheetCompiler::compile)
//! time) paired with its duration in seconds.
//!
//! Both the `animation` shorthand (`animation: pulse 1s ease-in`) and the
//! longhands (`animation-name: pulse; animation-duration: 1s`) are recognised;
//! other `animation-*` longhands (iteration count, direction, fill mode, …) are
//! not modelled and are ignored.

use crate::parser::{ParsedDeclaration, ParsedValue};

use obscur_identity::InternedSymbol;

use super::property_compiler::narrow_f32;
use super::stylesheet_compiler::CssSymbolInterner;

/// A single `@keyframes` reference declared by a style rule: the interned
/// animation name and its duration in seconds.
///
/// The name is interned in the stylesheet's shared [`CssSymbolInterner`], so it
/// compares equal to the matching [`CompiledKeyframes`](super::CompiledKeyframes)
/// name and can be resolved to that rule's tracks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationRef {
    /// The interned `@keyframes` name this rule plays.
    pub name: InternedSymbol,
    /// The animation duration, in seconds (`0.0` when unspecified).
    pub duration_secs: f32,
}

/// Lowers `animation`/`animation-*` declarations into [`AnimationRef`]s.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnimationCompiler;

impl AnimationCompiler {
    /// Creates an animation compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extracts every [`AnimationRef`] declared across `decls`, interning each
    /// animation name in `interner`.
    ///
    /// `animation` shorthand layers each contribute one ref (name + that layer's
    /// duration); bare `animation-name` idents each contribute one ref carrying
    /// the `animation-duration` longhand's value (or `0` when absent). An
    /// `animation-name: none` layer is skipped.
    #[must_use]
    pub fn compile(
        &self,
        decls: &[ParsedDeclaration],
        interner: &mut CssSymbolInterner,
    ) -> Vec<AnimationRef> {
        let mut shorthand: Vec<AnimationRef> = Vec::new();
        let mut long_names: Vec<InternedSymbol> = Vec::new();
        let mut long_duration: f32 = 0.0;

        for decl in decls {
            let name = decl.name.to_ascii_lowercase();
            match name.as_str() {
                "animation" => {
                    for layer in split_layers(&decl.value) {
                        if let Some(reference) = compile_shorthand_layer(layer, interner) {
                            shorthand.push(reference);
                        }
                    }
                }
                "animation-name" => {
                    for value in &decl.value {
                        if let ParsedValue::Keyword(keyword) = value
                            && !keyword.eq_ignore_ascii_case("none")
                        {
                            long_names.push(interner.intern(keyword));
                        }
                    }
                }
                "animation-duration" => {
                    if let Some(secs) = first_time_secs(&decl.value) {
                        long_duration = secs;
                    }
                }
                _ => {}
            }
        }

        let mut out = shorthand;
        for name in long_names {
            out.push(AnimationRef {
                name,
                duration_secs: long_duration,
            });
        }
        out
    }
}

/// Splits a component-value list into comma-separated layers.
fn split_layers(values: &[ParsedValue]) -> Vec<&[ParsedValue]> {
    values
        .split(|value| matches!(value, ParsedValue::Comma))
        .filter(|layer| !layer.is_empty())
        .collect()
}

/// Compiles one `animation` shorthand layer into an [`AnimationRef`], or [`None`]
/// when the layer names no animation.
///
/// The first keyword that is not a recognised timing-function/animation keyword
/// is taken as the animation name; the first time dimension is the duration.
fn compile_shorthand_layer(
    layer: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<AnimationRef> {
    let mut name: Option<InternedSymbol> = None;
    let mut duration: f32 = 0.0;
    for component in layer {
        match component {
            ParsedValue::Keyword(keyword) if !is_reserved_keyword(keyword) => {
                if name.is_none() {
                    name = Some(interner.intern(keyword));
                }
            }
            ParsedValue::Dimension { value, unit } => {
                if duration == 0.0
                    && let Some(secs) = time_secs(*value, unit)
                {
                    duration = secs;
                }
            }
            _ => {}
        }
    }
    name.map(|name| AnimationRef {
        name,
        duration_secs: duration,
    })
}

/// Returns the first time dimension in `values`, in seconds.
fn first_time_secs(values: &[ParsedValue]) -> Option<f32> {
    values.iter().find_map(|value| match value {
        ParsedValue::Dimension { value, unit } => time_secs(*value, unit),
        _ => None,
    })
}

/// Converts a `s`/`ms` dimension to seconds, or [`None`] for any other unit.
fn time_secs(value: f64, unit: &str) -> Option<f32> {
    if unit.eq_ignore_ascii_case("s") {
        Some(narrow_f32(value))
    } else if unit.eq_ignore_ascii_case("ms") {
        Some(narrow_f32(value) / 1000.0)
    } else {
        None
    }
}

/// Returns whether `keyword` is a reserved `animation`/timing keyword (so it is
/// not mistaken for an animation *name* in the shorthand).
fn is_reserved_keyword(keyword: &str) -> bool {
    const RESERVED: &[&str] = &[
        "none",
        "linear",
        "ease",
        "ease-in",
        "ease-out",
        "ease-in-out",
        "infinite",
        "normal",
        "reverse",
        "alternate",
        "alternate-reverse",
        "forwards",
        "backwards",
        "both",
        "running",
        "paused",
    ];
    RESERVED.iter().any(|r| keyword.eq_ignore_ascii_case(r))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::StylesheetParser;
    use crate::source::CssSource;

    fn decls(text: &str) -> Vec<ParsedDeclaration> {
        // Wrap the declarations in a throwaway rule so the stylesheet parser
        // yields them as a style rule's declaration list.
        let wrapped = format!("x {{ {text} }}");
        let src = CssSource::anonymous(wrapped.as_str());
        let sheet = StylesheetParser::new(&src).parse();
        match &sheet.rules[0] {
            crate::parser::ParsedRule::Style(style) => style.declarations.clone(),
            _ => panic!("expected a style rule"),
        }
    }

    fn compile(text: &str) -> (Vec<AnimationRef>, CssSymbolInterner) {
        let mut interner = CssSymbolInterner::new();
        let refs = AnimationCompiler::new().compile(&decls(text), &mut interner);
        (refs, interner)
    }

    #[test]
    fn shorthand_name_and_duration() {
        let (refs, interner) = compile("animation: pulse 1s ease-in");
        assert_eq!(refs.len(), 1);
        assert_eq!(interner.resolve(refs[0].name), Some("pulse"));
        assert_eq!(refs[0].duration_secs, 1.0);
    }

    #[test]
    fn shorthand_milliseconds() {
        let (refs, _) = compile("animation: spin 250ms");
        assert_eq!(refs[0].duration_secs, 0.25);
    }

    #[test]
    fn longhands_name_and_duration() {
        let (refs, interner) = compile("animation-name: fade; animation-duration: 2s");
        assert_eq!(refs.len(), 1);
        assert_eq!(interner.resolve(refs[0].name), Some("fade"));
        assert_eq!(refs[0].duration_secs, 2.0);
    }

    #[test]
    fn animation_name_none_is_skipped() {
        let (refs, _) = compile("animation-name: none");
        assert!(refs.is_empty());
    }

    #[test]
    fn no_animation_declaration_yields_nothing() {
        let (refs, _) = compile("color: red");
        assert!(refs.is_empty());
    }

    #[test]
    fn shorthand_reserved_keywords_are_not_names() {
        // `infinite`/`ease-out` must not be mistaken for the animation name.
        let (refs, interner) = compile("animation: ease-out 1s glow infinite");
        assert_eq!(refs.len(), 1);
        assert_eq!(interner.resolve(refs[0].name), Some("glow"));
        assert_eq!(refs[0].duration_secs, 1.0);
    }
}
