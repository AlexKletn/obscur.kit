#![doc = include_str!("inline_style_compiler.doc/_module.md")]

use obscur_style::matching::InlineStyle;

use crate::error::{CssParseError, UnsupportedCssFeature};
use crate::source::CssSource;

use crate::parser::StylesheetParser;

use super::StylesheetCompiler;

#[doc = include_str!("inline_style_compiler.doc/CompiledInlineStyle.md")]
#[derive(Debug, Clone, Default)]
pub struct CompiledInlineStyle {
    #[doc = include_str!("inline_style_compiler.doc/style.md")]
    pub style: InlineStyle,
    #[doc = include_str!("inline_style_compiler.doc/unsupported.md")]
    pub unsupported: Vec<UnsupportedCssFeature>,
    #[doc = include_str!("inline_style_compiler.doc/errors.md")]
    pub errors: Vec<CssParseError>,
}

#[doc = include_str!("inline_style_compiler.doc/compile_inline_style.md")]
#[must_use]
pub fn compile_inline_style(text: &str) -> CompiledInlineStyle {
    // Wrapped in a rule and put through the ordinary compiler rather than given
    // a parser of its own. A `style` attribute is a declaration block and
    // nothing else, so everything an authored block gets — shorthand expansion,
    // `calc()`, vendor namespacing, the diagnostics — it gets too, and none of
    // it can drift away from the sheets it competes with.
    //
    // The selector is synthetic and its specificity is discarded: what comes
    // back is a list of declarations, which carry none. An inline declaration
    // outranks a stylesheet by ORIGIN, never by out-specifying it.
    let wrapped = format!("*{{{text}}}");
    let parsed = StylesheetParser::new(&CssSource::anonymous(wrapped.as_str())).parse();
    let compiled = StylesheetCompiler::new().compile(&parsed);
    let rule = compiled.matchable.rules().first();
    let declarations = rule.map(|rule| rule.block().to_vec()).unwrap_or_default();
    // `style="--brand: #f00"` is a declaration the cascade has to see: an inline
    // custom property outranks every sheet that names the same one, and a block
    // that carried only the longhands would silently drop it.
    let custom_properties = rule
        .map(|rule| rule.custom_properties().to_vec())
        .unwrap_or_default();
    CompiledInlineStyle {
        style: InlineStyle::new(declarations).with_custom_properties(custom_properties),
        unsupported: compiled.unsupported,
        errors: compiled.errors,
    }
}

#[cfg(test)]
mod tests {
    use super::compile_inline_style;
    use obscur_style::cascade::ImportantFlag;
    use obscur_style::matching::BlockDeclaration;
    use obscur_style::property::{PropertyId, PropertyValue};

    fn properties(text: &str) -> Vec<PropertyId> {
        compile_inline_style(text)
            .style
            .declarations()
            .iter()
            .map(BlockDeclaration::property)
            .collect()
    }

    #[test]
    fn a_block_of_declarations_compiles_to_the_same_declarations() {
        let compiled = compile_inline_style("opacity:0.5;z-index:3");
        assert_eq!(
            properties("opacity:0.5;z-index:3"),
            vec![PropertyId::Opacity, PropertyId::ZIndex]
        );
        assert!(compiled.errors.is_empty());
        assert!(compiled.unsupported.is_empty());
    }

    #[test]
    fn a_shorthand_expands_exactly_as_it_would_in_a_sheet() {
        // The reason this goes through the ordinary compiler instead of a
        // parser of its own: `style="margin:4px"` has to mean what
        // `margin:4px` means everywhere else, and it cannot drift.
        let ids = properties("margin:4px");
        assert_eq!(ids.len(), 4, "four longhands, got {ids:?}");
        assert!(ids.contains(&PropertyId::MarginTop));
        assert!(ids.contains(&PropertyId::MarginLeft));
    }

    #[test]
    fn important_survives_into_the_block() {
        // It has to: §4.14 puts every important declaration above every normal
        // one, so losing the flag here would silently demote the declaration.
        let compiled = compile_inline_style("opacity:0.5 !important");
        assert_eq!(
            compiled.style.declarations()[0].important_flag(),
            ImportantFlag::Important
        );
    }

    #[test]
    fn a_trailing_semicolon_and_stray_whitespace_are_fine() {
        assert_eq!(properties("  opacity : 0.5 ; "), vec![PropertyId::Opacity]);
    }

    #[test]
    fn an_empty_attribute_declares_nothing() {
        assert!(compile_inline_style("").style.is_empty());
        assert!(compile_inline_style("   ").style.is_empty());
    }

    #[test]
    fn one_bad_declaration_does_not_take_the_others_with_it() {
        // §1.6 localized failure: the block keeps what parsed. A `style` that
        // dropped everything because one property was misspelled would be a
        // much worse failure than the misspelling.
        let compiled = compile_inline_style("opacity:0.5;float:left;z-index:3");
        let ids: Vec<_> = compiled
            .style
            .declarations()
            .iter()
            .map(BlockDeclaration::property)
            .collect();
        assert_eq!(ids, vec![PropertyId::Opacity, PropertyId::ZIndex]);
        assert_eq!(compiled.unsupported.len(), 1, "and the fault is reported");
    }

    #[test]
    fn a_closing_brace_cannot_escape_into_the_stylesheet_grammar() {
        // The attribute is wrapped in a synthetic rule, so an author who writes
        // a `}` must not be able to close it and start writing rules of their
        // own — a `style` attribute that could add a global selector would be a
        // hole, not a feature.
        let compiled = compile_inline_style("opacity:0.5} *{z-index:99}");
        let ids: Vec<_> = compiled
            .style
            .declarations()
            .iter()
            .map(BlockDeclaration::property)
            .collect();
        assert!(
            !ids.contains(&PropertyId::ZIndex),
            "the second rule did not become part of this block: {ids:?}"
        );
    }

    #[test]
    fn a_custom_property_in_the_attribute_reaches_the_block() {
        // `style="--brand: #f00"` is how an author themes ONE element, and the
        // block is the only channel it has: dropping it here would make an
        // inline custom property silently inert.
        use obscur_identity::InternedSymbol;
        use obscur_style::property::StyleColor;

        let compiled = compile_inline_style("--brand: #ff0000; color: var(--brand)");
        let custom = compiled.style.custom_properties();
        assert_eq!(custom.len(), 1);
        assert_eq!(custom[0].name(), InternedSymbol::named("--brand"));
        assert_eq!(
            custom[0].value(),
            &PropertyValue::Color(StyleColor::rgba(1.0, 0.0, 0.0, 1.0))
        );
        // And it is the same symbol the reading declaration asks for, even
        // though a `style` attribute is compiled by its own compiler instance.
        assert_eq!(
            compiled.style.declarations()[0].value(),
            &PropertyValue::CustomRef(InternedSymbol::named("--brand"))
        );
    }

    #[test]
    fn the_later_of_two_declarations_wins_by_order_not_by_luck() {
        let compiled = compile_inline_style("opacity:0.25;opacity:0.75");
        let last = compiled.style.declarations().last().expect("two of them");
        assert_eq!(last.value(), &PropertyValue::Number(0.75));
    }
}
