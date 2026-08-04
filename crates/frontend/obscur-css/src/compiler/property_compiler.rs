#![doc = include_str!("property_compiler.doc/_module.md")]

use super::background_lowering::{
    expand_background, lower_background_image, lower_background_position, lower_background_repeat,
    lower_background_size, lower_object_fit,
};
use super::border_shorthand::{
    BORDER_SIDES, border_side_shorthand, expand_border, expand_border_color, expand_border_style,
    expand_outline,
};
use super::filter_lowering::{lower_backdrop_filter, lower_filter};
use super::font_lowering::{lower_font_family, lower_font_weight};
use super::grid_placement::{lower_grid_auto_flow, lower_grid_line};
use super::grid_track_lowering::{
    lower_grid_auto_tracks, lower_grid_template, lower_grid_template_areas,
};
use super::longhand_catalog::{is_length_longhand, layout_keyword_mapper, property_id_from_name};
use super::shadow_lowering::lower_box_shadow;
use super::shorthand_expansion::{
    box_shorthand_sides, expand_box_shorthand, expand_flex_shorthand, expand_grid_area_shorthand,
    expand_grid_line_shorthand, expand_pair_shorthand, grid_line_shorthand_sides,
    logical_pair_shorthand_sides, place_shorthand_sides,
};
use super::stylesheet_compiler::CssSymbolInterner;
use super::transform_lowering::{lower_transform, lower_transform_origin};
use super::value_lowering::{css_wide_keyword, lower_value};
use super::vendor_prefix::{VendorNamespace, vendor_namespace};
use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{ParsedDeclaration, ParsedValue};
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{CustomProperty, Length, PropertyId, PropertyValue, layout_keyword};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

#[doc = include_str!("property_compiler.doc/CompiledDeclaration.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum CompiledDeclaration {
    #[doc = include_str!("property_compiler.doc/Standard.md")]
    Standard(Declaration),
    #[doc = include_str!("property_compiler.doc/Custom.md")]
    Custom(CustomProperty),
    #[doc = include_str!("property_compiler.doc/Unsupported.md")]
    Unsupported(UnsupportedCssFeature),
}

#[doc = include_str!("property_compiler.doc/PropertyCompiler.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct PropertyCompiler;

impl PropertyCompiler {
    #[doc = include_str!("property_compiler.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("property_compiler.doc/compile.md")]
    #[must_use]
    pub fn compile(
        &self,
        decl: &ParsedDeclaration,
        specificity: SelectorSpecificity,
        origin: CascadeOrigin,
        source_order: u32,
        selector: Option<SelectorId>,
        interner: &mut CssSymbolInterner,
    ) -> CompiledDeclaration {
        let name = &*decl.name;
        if name.starts_with("--") {
            // A custom property keeps its value verbatim (best-effort); an
            // unlowerable value degrades to `initial` rather than being dropped.
            let value = lower_value(&decl.value, interner).unwrap_or(PropertyValue::Initial);
            let symbol = interner.intern(name);
            return CompiledDeclaration::Custom(CustomProperty::new(symbol, value));
        }
        // Another vendor's property is passed over in SILENCE — recorded, but
        // as a note rather than a fault. That is the entire contract of a
        // prefix: a renderer that does not know it ignores it, which is what
        // lets one stylesheet serve renderers that do not know each other.
        // Reporting it as unsupported would make a portable sheet look broken.
        // Our own namespace gets no such indulgence: `-obscur-paintr` is a typo
        // in a name this compiler is supposed to know, and it falls through to
        // the ordinary unknown-property path below.
        if let Some(VendorNamespace::Foreign(_)) = vendor_namespace(name) {
            return CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
                UnsupportedFeatureKind::VendorProperty,
                name,
                decl.span,
            ));
        }
        let Some(property) = property_id_from_name(name) else {
            return CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
                UnsupportedFeatureKind::Property,
                name,
                decl.span,
            ));
        };
        match lower_for_property(property, &decl.value, interner) {
            Some(value) => {
                let important = if decl.important {
                    ImportantFlag::Important
                } else {
                    ImportantFlag::Normal
                };
                CompiledDeclaration::Standard(Declaration::new(
                    property,
                    value,
                    important,
                    origin,
                    specificity,
                    source_order,
                    selector,
                ))
            }
            None => CompiledDeclaration::Unsupported(UnsupportedCssFeature::new(
                value_unsupported_kind(&decl.value),
                name,
                decl.span,
            )),
        }
    }

    #[doc = include_str!("property_compiler.doc/compile_all.md")]
    #[must_use]
    pub fn compile_all(
        &self,
        decl: &ParsedDeclaration,
        specificity: SelectorSpecificity,
        origin: CascadeOrigin,
        source_order: u32,
        selector: Option<SelectorId>,
        interner: &mut CssSymbolInterner,
    ) -> Vec<CompiledDeclaration> {
        if let Some(sides) = box_shorthand_sides(&decl.name) {
            return expand_box_shorthand(
                decl,
                sides,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        if decl.name.eq_ignore_ascii_case("background") {
            return expand_background(decl, specificity, origin, source_order, selector, interner);
        }
        if decl.name.eq_ignore_ascii_case("border-color") {
            return expand_border_color(
                decl,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        if decl.name.eq_ignore_ascii_case("border-style") {
            return expand_border_style(decl, specificity, origin, source_order, selector);
        }
        if decl.name.eq_ignore_ascii_case("border") {
            return expand_border(
                decl,
                BORDER_SIDES,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        if decl.name.eq_ignore_ascii_case("outline") {
            return expand_outline(decl, specificity, origin, source_order, selector, interner);
        }
        if let Some(side) = border_side_shorthand(&decl.name) {
            return expand_border(
                decl,
                side,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        if let Some((start, end)) = grid_line_shorthand_sides(&decl.name) {
            return expand_grid_line_shorthand(
                decl,
                start,
                end,
                specificity,
                origin,
                source_order,
                selector,
            );
        }
        if decl.name.eq_ignore_ascii_case("flex") {
            return expand_flex_shorthand(
                decl,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        if decl.name.eq_ignore_ascii_case("grid-area") {
            return expand_grid_area_shorthand(decl, specificity, origin, source_order, selector);
        }
        // The two-longhand shorthands: `place-*` (`<align> <justify>`) and the
        // logical box-model pairs (`<start> <end>`). Different families, identical
        // shape — see `expand_pair_shorthand`.
        if let Some((first, second)) =
            place_shorthand_sides(&decl.name).or_else(|| logical_pair_shorthand_sides(&decl.name))
        {
            return expand_pair_shorthand(
                decl,
                first,
                second,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        vec![self.compile(decl, specificity, origin, source_order, selector, interner)]
    }
}

#[doc = include_str!("property_compiler.doc/narrow_f32.md")]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn narrow_f32(value: f64) -> f32 {
    value as f32
}

#[doc = include_str!("property_compiler.doc/lower_for_property.md")]
pub(super) fn lower_for_property(
    property: PropertyId,
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    if property == PropertyId::Content {
        // `content: "…"` (§4.16 generated content). Only the plain string form is
        // modelled — the quotes are already stripped by the tokenizer — so a
        // single string component lowers to a `String` value; anything else
        // (keywords, `attr()`, counters) is unsupported and dropped.
        if let [ParsedValue::String(text)] = values {
            return Some(PropertyValue::String(text.clone()));
        }
        return None;
    }
    if property == PropertyId::Painter {
        return lower_painter(values, interner);
    }
    if property == PropertyId::FontFamily {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_font_family(values);
    }
    if property == PropertyId::FontWeight {
        return lower_font_weight(values);
    }
    if property == PropertyId::BackgroundImage {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_background_image(values);
    }
    if property == PropertyId::BackgroundSize {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_background_size(values);
    }
    if property == PropertyId::BackgroundPosition {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_background_position(values);
    }
    if property == PropertyId::BackgroundRepeat {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_background_repeat(values);
    }
    if property == PropertyId::ObjectFit {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_object_fit(values);
    }
    if property == PropertyId::ObjectPosition {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        // `object-position` is the same `<position>` grammar as
        // `background-position`; reuse its lowering (CSS Images 3 §5.2).
        return lower_background_position(values);
    }
    if property == PropertyId::BoxShadow {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_box_shadow(values);
    }
    if property == PropertyId::Filter {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_filter(values);
    }
    if property == PropertyId::BackdropFilter {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_backdrop_filter(values);
    }
    if property == PropertyId::Transform {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_transform(values);
    }
    if property == PropertyId::TransformOrigin {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_transform_origin(values);
    }
    if property == PropertyId::ZIndex {
        return lower_z_index(values);
    }
    if matches!(
        property,
        PropertyId::GridColumnStart
            | PropertyId::GridColumnEnd
            | PropertyId::GridRowStart
            | PropertyId::GridRowEnd
    ) {
        return lower_grid_line(values);
    }
    if matches!(
        property,
        PropertyId::GridTemplateColumns | PropertyId::GridTemplateRows
    ) {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_grid_template(values);
    }
    if property == PropertyId::GridTemplateAreas {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_grid_template_areas(values);
    }
    if property == PropertyId::GridAutoFlow {
        return lower_grid_auto_flow(values);
    }
    if matches!(
        property,
        PropertyId::GridAutoColumns | PropertyId::GridAutoRows
    ) {
        if let Some(ParsedValue::Keyword(keyword)) = values.first()
            && let Some(wide) = css_wide_keyword(keyword)
            && values.len() == 1
        {
            return Some(wide);
        }
        return lower_grid_auto_tracks(values);
    }
    if let Some(mapper) = layout_keyword_mapper(property)
        && let Some(ParsedValue::Keyword(keyword)) = values.first()
    {
        if let Some(wide) = css_wide_keyword(keyword) {
            return Some(wide);
        }
        return Some(match mapper(keyword) {
            Some(symbol) => PropertyValue::Keyword(symbol),
            None => PropertyValue::Keyword(interner.intern(keyword)),
        });
    }
    // A unitless `0` is a valid length for any length-typed longhand — `0` means
    // `0px` (CSS Values §4.5.3, "the unit may be omitted for zero lengths"). The
    // generic `lower_value` path would otherwise lower it to a `PropertyValue::
    // Number`, which a length property ignores (so `top: 0` would read as unset
    // instead of `0px`, unlike `top: 0px` or the `inset: 0` shorthand which the
    // box-shorthand path already handles via `box_side_length`).
    if is_length_longhand(property)
        && let [ParsedValue::Number(n)] = values
        && *n == 0.0
    {
        return Some(PropertyValue::Length(Length::px(0.0)));
    }
    lower_value(values, interner)
}

#[doc = include_str!("property_compiler.doc/lower_painter.md")]
fn lower_painter(
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    match values {
        // A quoted name, for a painter registered under something that is not a
        // CSS identifier — `"chart.sparkline"`, say.
        [ParsedValue::String(text)] => Some(PropertyValue::Name(text.clone())),
        [ParsedValue::Keyword(keyword)] => {
            if let Some(wide) = css_wide_keyword(keyword) {
                return Some(wide);
            }
            // `none` is the absence of a painter, and it is a keyword rather
            // than a name so that a rule can UNBIND what a lower origin bound.
            // A painter registered under the name "none" would be unreachable,
            // which is the same trade every CSS property with a `none` value
            // makes.
            if keyword.eq_ignore_ascii_case("none") {
                return Some(PropertyValue::Keyword(interner.intern("none")));
            }
            Some(PropertyValue::Name(keyword.as_ref().into()))
        }
        _ => None,
    }
}

#[doc = include_str!("property_compiler.doc/lower_z_index.md")]
fn lower_z_index(values: &[ParsedValue]) -> Option<PropertyValue> {
    match values.first()? {
        ParsedValue::Number(n) if n.fract() == 0.0 => Some(PropertyValue::Number(narrow_f32(*n))),
        ParsedValue::Keyword(keyword) => {
            if let Some(wide) = css_wide_keyword(keyword) {
                return Some(wide);
            }
            layout_keyword::z_index_from_name(keyword).map(PropertyValue::Keyword)
        }
        _ => None,
    }
}

#[doc = include_str!("property_compiler.doc/value_unsupported_kind.md")]
pub(super) fn value_unsupported_kind(values: &[ParsedValue]) -> UnsupportedFeatureKind {
    match values.first() {
        Some(ParsedValue::Function { .. }) => UnsupportedFeatureKind::Function,
        Some(ParsedValue::Dimension { .. }) => UnsupportedFeatureKind::Unit,
        _ => UnsupportedFeatureKind::Property,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::parser::{ParsedRule, PropertyParser, StylesheetParser};
    use crate::source::CssSource;
    use obscur_style::property::StyleColor;

    pub(crate) fn decl(text: &str) -> ParsedDeclaration {
        let src = CssSource::anonymous(text);
        PropertyParser::new().parse(&src).unwrap()
    }

    pub(crate) fn compile(text: &str) -> (CompiledDeclaration, CssSymbolInterner) {
        let mut interner = CssSymbolInterner::new();
        let parsed = decl(text);
        let result = PropertyCompiler::new().compile(
            &parsed,
            SelectorSpecificity::ZERO,
            CascadeOrigin::GlobalApp,
            0,
            None,
            &mut interner,
        );
        (result, interner)
    }

    pub(crate) fn standard(text: &str) -> (Declaration, CssSymbolInterner) {
        match compile(text) {
            (CompiledDeclaration::Standard(d), i) => (d, i),
            (other, _) => panic!("expected standard declaration, got {other:?}"),
        }
    }

    pub(crate) fn value_of(text: &str) -> PropertyValue {
        standard(text).0.value().clone()
    }

    pub(crate) fn compile_all(text: &str) -> Vec<CompiledDeclaration> {
        let mut interner = CssSymbolInterner::new();
        let parsed = decl(text);
        PropertyCompiler::new().compile_all(
            &parsed,
            SelectorSpecificity::ZERO,
            CascadeOrigin::GlobalApp,
            0,
            None,
            &mut interner,
        )
    }

    pub(crate) fn standard_of(compiled: &CompiledDeclaration) -> &Declaration {
        match compiled {
            CompiledDeclaration::Standard(d) => d,
            other => panic!("expected standard declaration, got {other:?}"),
        }
    }

    #[doc = include_str!("property_compiler.doc/pairs.md")]
    pub(crate) fn pairs(compiled: &[CompiledDeclaration]) -> Vec<(PropertyId, PropertyValue)> {
        compiled
            .iter()
            .map(|c| {
                let d = standard_of(c);
                (d.property(), d.value().clone())
            })
            .collect()
    }

    #[test]
    fn unknown_property_is_unsupported() {
        let (result, _) = compile("float: left");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected unsupported");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::Property);
        assert_eq!(feature.name(), "float");
    }

    #[test]
    fn another_vendors_property_is_ignored_rather_than_faulted() {
        // The contract of a prefix, and the reason one stylesheet can serve
        // renderers that do not know each other: a renderer that does not know
        // a vendor's property ignores it. Reporting it would make a portable
        // sheet look broken.
        for text in [
            "-webkit-box-reflect: below",
            "-moz-osx-font-smoothing: grayscale",
            "-someothertoolkit-glow: 4px",
        ] {
            let (result, _) = compile(text);
            let CompiledDeclaration::Unsupported(feature) = result else {
                panic!("expected {text} to be passed over");
            };
            assert_eq!(
                feature.kind(),
                UnsupportedFeatureKind::VendorProperty,
                "{text}"
            );
        }
    }

    #[test]
    fn another_vendors_prefix_is_not_an_alias_for_one_of_ours() {
        // Spec §4.13 excludes vendor hacks, and this is the one that matters:
        // `-webkit-filter` does NOT become `filter`. Accepting foreign prefixes
        // as aliases is how a renderer ends up implementing another vendor's
        // history.
        let (result, _) = compile("-webkit-filter: blur(2px)");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected it to be passed over");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::VendorProperty);
    }

    #[test]
    fn a_typo_in_our_own_namespace_is_reported() {
        // Our own namespace gets no indulgence: `-obscur-paintr` is a typo in a
        // name this compiler is supposed to know, and silence there would let a
        // misspelled rule sit in a stylesheet doing nothing.
        let (result, _) = compile("-obscur-paintr: clock");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected unsupported");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::Property);
        assert_eq!(feature.name(), "-obscur-paintr");
    }

    #[test]
    fn lowers_a_painter_name_as_text_not_as_a_symbol() {
        // As TEXT, deliberately: an interned keyword's symbol is minted per
        // stylesheet, so the same word compiled in two sheets would be two
        // different symbols and neither would match the painter registry.
        let (result, _) = compile("-obscur-painter: sparkline");
        let CompiledDeclaration::Standard(decl) = result else {
            panic!("expected a standard declaration");
        };
        assert_eq!(decl.property(), PropertyId::Painter);
        assert_eq!(decl.value(), &PropertyValue::Name("sparkline".into()));

        let (quoted, _) = compile("-obscur-painter: \"chart.sparkline\"");
        let CompiledDeclaration::Standard(decl) = quoted else {
            panic!("expected a standard declaration");
        };
        assert_eq!(decl.value(), &PropertyValue::Name("chart.sparkline".into()));
    }

    #[test]
    fn a_painter_of_none_unbinds_rather_than_naming_a_painter_called_none() {
        let (result, interner) = compile("-obscur-painter: none");
        let CompiledDeclaration::Standard(decl) = result else {
            panic!("expected a standard declaration");
        };
        let PropertyValue::Keyword(symbol) = decl.value() else {
            panic!("expected a keyword, got {:?}", decl.value());
        };
        assert_eq!(interner.resolve(*symbol), Some("none"));
    }

    #[test]
    fn lowers_custom_property() {
        let (result, interner) = compile("--brand: #fff");
        let CompiledDeclaration::Custom(custom) = result else {
            panic!("expected custom property");
        };
        assert_eq!(interner.resolve(custom.name()), Some("--brand"));
        assert_eq!(
            custom.value(),
            &PropertyValue::Color(StyleColor::rgba(1.0, 1.0, 1.0, 1.0))
        );
    }

    #[test]
    fn important_flag_is_carried() {
        let (normal, _) = standard("color: red");
        assert_eq!(normal.important(), ImportantFlag::Normal);
        let (important, _) = standard("color: red !important");
        assert_eq!(important.important(), ImportantFlag::Important);
    }

    #[test]
    fn cascade_context_is_recorded() {
        let mut interner = CssSymbolInterner::new();
        let parsed = decl("width: 10px");
        let selector = SelectorId::from_raw(9);
        let result = PropertyCompiler::new().compile(
            &parsed,
            SelectorSpecificity::new(1, 2, 3),
            CascadeOrigin::Theme,
            7,
            Some(selector),
            &mut interner,
        );
        let CompiledDeclaration::Standard(d) = result else {
            panic!("expected standard");
        };
        assert_eq!(d.origin(), CascadeOrigin::Theme);
        assert_eq!(d.specificity(), SelectorSpecificity::new(1, 2, 3));
        assert_eq!(d.source_order(), 7);
        assert_eq!(d.selector(), Some(selector));
    }

    #[test]
    fn full_declaration_end_to_end() {
        let src = CssSource::anonymous("a { color: #00ff00 }");
        let sheet = StylesheetParser::new(&src).parse();
        let ParsedRule::Style(rule) = &sheet.rules[0] else {
            panic!("expected style rule");
        };
        let mut interner = CssSymbolInterner::new();
        let result = PropertyCompiler::new().compile(
            &rule.declarations[0],
            SelectorSpecificity::ZERO,
            CascadeOrigin::GlobalApp,
            0,
            None,
            &mut interner,
        );
        assert!(matches!(
            result,
            CompiledDeclaration::Standard(d)
                if *d.value() == PropertyValue::Color(StyleColor::rgba(0.0, 1.0, 0.0, 1.0))
        ));
    }

    // ---- P4b: box-model / sizing / flex / font longhands ----

    #[test]
    fn z_index_lowers_integer_and_auto() {
        // `z-index` is resolved a longhand name.
        assert_eq!(property_id_from_name("z-index"), Some(PropertyId::ZIndex));
        // A positive integer lowers to a bare number.
        assert_eq!(value_of("z-index: 5"), PropertyValue::Number(5.0));
        // A negative integer is valid and lowers to a negative number.
        assert_eq!(value_of("z-index: -1"), PropertyValue::Number(-1.0));
        // `auto` lowers to the reserved cross-crate handle.
        assert_eq!(
            value_of("z-index: auto"),
            PropertyValue::Keyword(layout_keyword::Z_INDEX_AUTO)
        );
        // CSS-wide keywords still take priority.
        assert_eq!(value_of("z-index: inherit"), PropertyValue::Inherit);
    }

    #[test]
    fn z_index_non_integer_is_unsupported() {
        // A non-integral number is not a valid `z-index`; it degrades rather than
        // being dropped.
        let (result, _) = compile("z-index: 1.5");
        assert!(matches!(result, CompiledDeclaration::Unsupported(_)));
    }

    #[test]
    fn compile_all_wraps_a_single_longhand() {
        let decls = compile_all("width: 10px");
        assert_eq!(decls.len(), 1);
        assert_eq!(standard_of(&decls[0]).property(), PropertyId::Width);
    }
}
