#![doc = include_str!("../../docs/api/compiler/property_compiler.doc/_module.md")]

use super::CssSymbolInterner;
use super::background_lowering::{
    expand_background, lower_background_image, lower_background_position, lower_background_repeat,
    lower_background_size, lower_object_fit, lower_object_position,
};
use super::border_shorthand::{
    BORDER_SIDES, border_side_shorthand, expand_border, expand_border_color, expand_border_style,
    expand_outline,
};
use super::clip_path_lowering::lower_clip_path;
use super::custom_value::custom_value_text;
use super::filter_lowering::{lower_backdrop_filter, lower_filter};
use super::font_lowering::{lower_font_family, lower_font_weight};
use super::grid_placement::{lower_grid_auto_flow, lower_grid_line};
use super::grid_track_lowering::{
    lower_grid_auto_tracks, lower_grid_template, lower_grid_template_areas,
};
use super::longhand_catalog::{is_length_longhand, layout_keyword_mapper, property_id_from_name};
use super::shadow_lowering::lower_box_shadow;
use super::shorthand_expansion::{
    axis_pair_shorthand_sides, box_shorthand_sides, container_reference, contains_var,
    expand_box_shorthand, expand_container_shorthand, expand_flex_flow_shorthand,
    expand_flex_shorthand, expand_grid_area_shorthand, expand_grid_line_shorthand,
    expand_grid_shorthand, expand_pair_shorthand, grid_line_shorthand_sides,
    logical_pair_shorthand_sides, place_shorthand_sides, shorthand_components,
};
use super::transform_lowering::{lower_transform, lower_transform_origin};
use super::value_lowering::{css_wide_keyword, lower_value, sole_custom_ref};
use super::vendor_prefix::{VendorNamespace, vendor_namespace};
use crate::error::{UnsupportedCssFeature, UnsupportedFeatureKind};
use crate::parser::{ParsedDeclaration, ParsedValue};
use obscur_identity::InternedSymbol;
use obscur_style::cascade::{CascadeOrigin, Declaration, ImportantFlag};
use obscur_style::property::{CustomProperty, Length, PropertyId, PropertyValue, layout_keyword};
use obscur_style::selector::{SelectorId, SelectorSpecificity};

use super::CompiledDeclaration;

#[doc = include_str!("../../docs/api/compiler/property_compiler.doc/PropertyCompiler.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct PropertyCompiler;

impl PropertyCompiler {
    /// Creates a property compiler.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Lowers `decl` into a [`CompiledDeclaration`], resolving the cascade
    /// context (`specificity`, `origin`, `source_order`, `selector`) into the
    /// produced [`Declaration`] when the name is a known longhand.
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
            // Kept as TEXT, not lowered. A custom property's value is an
            // arbitrary token sequence (CSS Variables 1 §2) that matches no
            // grammar until `var()` puts it somewhere, so there is nothing here
            // to lower it against — and lowering it anyway kept only its first
            // component: `--font-body: Inter, sans-serif` became the one
            // keyword `Inter`, and `--shadow: 0 2px 4px #000` became `0`. The
            // grammar is applied at substitution instead, by the property the
            // value lands on (`custom_value::lower_custom_value`).
            //
            // The one value that is NOT text is a lone `var()`: a custom
            // property defined in terms of another is a reference the resolver
            // follows, and flattening the chain to text would leave a name
            // nothing resolves.
            let value = sole_custom_ref(&decl.value)
                .unwrap_or_else(|| PropertyValue::RawTokens(custom_value_text(&decl.value)));
            // Minted from the NAME, not from this sheet's interner. A compiler
            // is built per source, so its interner numbers from zero, and two
            // sheets that each declare a custom property would otherwise mint
            // the same symbol for two different names — the theme sheet's
            // `--brand` and the widget sheet's `--radius` colliding at symbol 0.
            // `var()` mints its reference the same way (`value_lowering`), which
            // is what lets one sheet declare what another one reads.
            return CompiledDeclaration::Custom(CustomProperty::new(
                InternedSymbol::named(name),
                value,
            ));
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

    #[doc = include_str!("../../docs/api/compiler/property_compiler.doc/compile_all.md")]
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
        if decl.name.eq_ignore_ascii_case("flex-flow") {
            return expand_flex_flow_shorthand(decl, specificity, origin, source_order, selector);
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
        if decl.name.eq_ignore_ascii_case("container") {
            return expand_container_shorthand(
                decl,
                specificity,
                origin,
                source_order,
                selector,
                interner,
            );
        }
        if decl.name.eq_ignore_ascii_case("grid-template") || decl.name.eq_ignore_ascii_case("grid")
        {
            return expand_grid_shorthand(decl, specificity, origin, source_order, selector);
        }
        if decl.name.eq_ignore_ascii_case("grid-area") {
            return expand_grid_area_shorthand(decl, specificity, origin, source_order, selector);
        }
        // The two-longhand shorthands: `place-*` (`<align> <justify>`), the
        // logical box-model pairs (`<start> <end>`), and `overflow`
        // (`<x> <y>`). Different families, identical shape — see
        // `expand_pair_shorthand`.
        if let Some((first, second)) = place_shorthand_sides(&decl.name)
            .or_else(|| logical_pair_shorthand_sides(&decl.name))
            .or_else(|| axis_pair_shorthand_sides(&decl.name))
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

/// Narrows parsed CSS numbers to f32, saturating values outside its finite range.
#[must_use]
pub(crate) fn narrow_f32(value: f64) -> f32 {
    obscur_geometry::saturating_f32(value)
}

fn lower_container_names(values: &[ParsedValue]) -> Option<PropertyValue> {
    if let [ParsedValue::Keyword(name)] = values {
        if let Some(wide) = css_wide_keyword(name) {
            return Some(wide);
        }
        if name.eq_ignore_ascii_case("none") {
            return Some(PropertyValue::Name("none".into()));
        }
    }
    let (first, rest) = values.split_first()?;
    let first = container_ident(first)?;
    if rest.is_empty() {
        return Some(PropertyValue::Name(first));
    }
    let mut names = Vec::with_capacity(values.len());
    names.push(first);
    for value in rest {
        names.push(container_ident(value)?);
    }
    Some(PropertyValue::ContainerNames(names.into()))
}

fn container_ident(value: &ParsedValue) -> Option<Box<str>> {
    let ParsedValue::Keyword(name) = value else {
        return None;
    };
    (css_wide_keyword(name).is_none()
        && ![
            "none",
            "and",
            "or",
            "not",
            "default",
            "revert",
            "revert-layer",
        ]
        .iter()
        .any(|reserved| name.eq_ignore_ascii_case(reserved)))
    .then(|| name.clone())
}

/// Parses a two-keyword baseline preference in either grammar order.
pub(super) fn baseline_position(values: &[ParsedValue]) -> Option<PropertyValue> {
    use obscur_style::property::layout_keyword::{ALIGN_BASELINE, ALIGN_LAST_BASELINE};

    let [ParsedValue::Keyword(a), ParsedValue::Keyword(b)] = values else {
        return None;
    };
    let preference = if a.eq_ignore_ascii_case("baseline") {
        b
    } else if b.eq_ignore_ascii_case("baseline") {
        a
    } else {
        return None;
    };
    let keyword = match preference.to_ascii_lowercase().as_str() {
        "first" => ALIGN_BASELINE,
        "last" => ALIGN_LAST_BASELINE,
        _ => return None,
    };
    Some(PropertyValue::Keyword(keyword))
}

#[doc = include_str!("../../docs/api/compiler/property_compiler.doc/lower_for_property.md")]
pub(super) fn lower_for_property(
    property: PropertyId,
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    // A `var()` cannot be judged against a grammar HERE: what it stands for does
    // not exist until the cascade has run, so every property below that parses
    // its value eagerly — `font-family`, `box-shadow`, `transform`, the grid
    // templates — used to reject the function and take the whole declaration
    // down with it. Carrying the reference defers the judgement to the one place
    // that can make it: the computed-value step substitutes, and drops the value
    // there if what came back does not fit this property.
    if let [value @ ParsedValue::Function { name, .. }] = values
        && name.eq_ignore_ascii_case("var")
    {
        use obscur_style::property::ContainerComponent;
        let reference = container_reference(value)?;
        return Some(match reference {
            ContainerComponent::Variable {
                name,
                fallback: None,
            } => PropertyValue::CustomRef(name),
            reference => PropertyValue::VariableTemplate(std::sync::Arc::from([reference])),
        });
    }
    if values.iter().any(contains_var) {
        return shorthand_components(values).map(PropertyValue::VariableTemplate);
    }
    if matches!(
        property,
        PropertyId::AlignItems
            | PropertyId::AlignSelf
            | PropertyId::JustifyItems
            | PropertyId::JustifySelf
    ) && let Some(baseline) = baseline_position(values)
    {
        return Some(baseline);
    }
    if matches!(
        property,
        PropertyId::ContainIntrinsicWidth
            | PropertyId::ContainIntrinsicHeight
            | PropertyId::ContainIntrinsicInlineSize
            | PropertyId::ContainIntrinsicBlockSize
    ) {
        let [value] = values else { return None };
        return match value {
            ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("none") => {
                Some(PropertyValue::Initial)
            }
            ParsedValue::Keyword(keyword) => css_wide_keyword(keyword),
            ParsedValue::Number(value) if *value == 0.0 => {
                Some(PropertyValue::Length(Length::px(0.0)))
            }
            ParsedValue::Dimension { value: number, .. } if *number >= 0.0 => {
                super::value_lowering::lower_component(value, interner)
            }
            ParsedValue::Function { .. } => {
                match super::value_lowering::lower_component(value, interner) {
                    Some(PropertyValue::Calc(expr)) if !expr.uses_percent() => {
                        Some(PropertyValue::Calc(expr))
                    }
                    _ => None,
                }
            }
            _ => None,
        };
    }
    if property == PropertyId::Content {
        return match values {
            [ParsedValue::String(text)] => Some(PropertyValue::String(text.clone())),
            [ParsedValue::Keyword(keyword)]
                if keyword.eq_ignore_ascii_case("none")
                    || keyword.eq_ignore_ascii_case("normal") =>
            {
                Some(PropertyValue::Name(keyword.clone()))
            }
            _ => None,
        };
    }
    if property == PropertyId::ContainerName {
        return lower_container_names(values);
    }
    if property == PropertyId::ContainerType {
        let [ParsedValue::Keyword(value)] = values else {
            return None;
        };
        if let Some(wide) = css_wide_keyword(value) {
            return Some(wide);
        }
        let name = value.to_ascii_lowercase();
        return matches!(name.as_str(), "normal" | "inline-size" | "size")
            .then_some(PropertyValue::Name(name.into_boxed_str()));
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
    if property == PropertyId::ClipPath {
        return lower_clip_path(values);
    }
    if property == PropertyId::BackgroundAttachment {
        if let [ParsedValue::Keyword(keyword)] = values
            && let Some(wide) = css_wide_keyword(keyword)
        {
            return Some(wide);
        }
        return values
            .split(|value| matches!(value, ParsedValue::Comma))
            .map(|segment| match segment {
                [ParsedValue::Keyword(keyword)] => {
                    super::background_lowering::background_attachment(keyword)
                }
                _ => None,
            })
            .collect::<Option<Box<[_]>>>()
            .map(PropertyValue::BackgroundAttachments);
    }
    if matches!(
        property,
        PropertyId::BackgroundClip | PropertyId::BackgroundOrigin
    ) {
        if let [ParsedValue::Keyword(keyword)] = values
            && let Some(wide) = css_wide_keyword(keyword)
        {
            return Some(wide);
        }
        return values
            .split(|value| matches!(value, ParsedValue::Comma))
            .map(|segment| match segment {
                [ParsedValue::Keyword(keyword)] => {
                    super::background_lowering::background_box(keyword)
                }
                _ => None,
            })
            .collect::<Option<Box<[_]>>>()
            .map(if property == PropertyId::BackgroundClip {
                PropertyValue::BackgroundClips
            } else {
                PropertyValue::BackgroundOrigins
            });
    }
    if property == PropertyId::BackgroundBlendMode {
        if let [ParsedValue::Keyword(keyword)] = values
            && let Some(wide) = css_wide_keyword(keyword)
        {
            return Some(wide);
        }
        return values
            .split(|value| matches!(value, ParsedValue::Comma))
            .map(|layer| match layer {
                [ParsedValue::Keyword(keyword)] => {
                    obscur_style::property::layout_keyword::mix_blend_mode_from_name(keyword)
                }
                _ => None,
            })
            .collect::<Option<Box<[_]>>>()
            .map(PropertyValue::BackgroundBlendModes);
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
        return lower_object_position(values);
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
    if property == PropertyId::Order {
        return match values {
            [ParsedValue::Number(n)] if n.fract() == 0.0 => {
                Some(PropertyValue::Number(narrow_f32(*n)))
            }
            [ParsedValue::Keyword(keyword)] => css_wide_keyword(keyword),
            _ => None,
        };
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
    if matches!(property, PropertyId::RowGap | PropertyId::ColumnGap) {
        return match values {
            [ParsedValue::Keyword(keyword)] if keyword.eq_ignore_ascii_case("normal") => {
                Some(PropertyValue::Length(Length::px(0.0)))
            }
            [ParsedValue::Keyword(keyword)] => css_wide_keyword(keyword),
            [ParsedValue::Number(number)] if *number == 0.0 => {
                Some(PropertyValue::Length(Length::px(0.0)))
            }
            [ParsedValue::Dimension { value, .. } | ParsedValue::Percentage(value)]
                if *value < 0.0 =>
            {
                None
            }
            [value] => match lower_value(std::slice::from_ref(value), interner)? {
                value @ (PropertyValue::Calc(_) | PropertyValue::Length(_)) => Some(value),
                _ => None,
            },
            _ => None,
        };
    }
    if matches!(property, PropertyId::MaxWidth | PropertyId::MaxHeight)
        && let [ParsedValue::Keyword(keyword)] = values
        && keyword.eq_ignore_ascii_case("none")
    {
        return Some(PropertyValue::Keyword(
            obscur_style::property::layout_keyword::SIZING_NONE,
        ));
    }
    if let Some(mapper) = layout_keyword_mapper(property)
        && let Some(ParsedValue::Keyword(keyword)) = values.first()
    {
        if let Some(wide) = css_wide_keyword(keyword) {
            return Some(wide);
        }
        // The mapper IS the grammar of these properties: their keywords are a
        // closed, reserved set, so a word it does not know is a word the
        // property does not have and the declaration is invalid (CSS Syntax 3
        // §9, "a declaration whose value does not match the property's grammar
        // is dropped"). Interning it instead minted a symbol no consumer can
        // recognise, and one consumer read it as a value: `OverflowAxes` takes
        // anything that is not `visible` or `clip` for clipping, so
        // `overflow-x: sideways` clipped the box. Judging it here — and not in
        // `custom_value::lower_custom_value`, which used to hold the only copy
        // of this rule — is what makes the literal and the `var()` spelling of
        // the same typo answer the same way.
        return mapper(keyword).map(PropertyValue::Keyword);
    }
    if is_length_longhand(property) {
        return lower_length_longhand(property, values, interner);
    }
    lower_value(values, interner)
}

/// Validates the full declaration and the literal numeric range before lowering a length. Math expressions retain their computed-value semantics.
fn lower_length_longhand(
    property: PropertyId,
    values: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<PropertyValue> {
    let [value] = values else {
        return None;
    };
    let sizing = matches!(
        property,
        PropertyId::Width
            | PropertyId::Height
            | PropertyId::MinWidth
            | PropertyId::MinHeight
            | PropertyId::MaxWidth
            | PropertyId::MaxHeight
            | PropertyId::FlexBasis
    );
    let signed = matches!(
        property,
        PropertyId::Top
            | PropertyId::Right
            | PropertyId::Bottom
            | PropertyId::Left
            | PropertyId::MarginTop
            | PropertyId::MarginRight
            | PropertyId::MarginBottom
            | PropertyId::MarginLeft
            | PropertyId::MarginInlineStart
            | PropertyId::MarginInlineEnd
            | PropertyId::MarginBlockStart
            | PropertyId::MarginBlockEnd
            | PropertyId::InsetInlineStart
            | PropertyId::InsetInlineEnd
            | PropertyId::InsetBlockStart
            | PropertyId::InsetBlockEnd
            | PropertyId::OutlineOffset
            | PropertyId::LetterSpacing
            | PropertyId::WordSpacing
    );
    match value {
        ParsedValue::Keyword(word) => {
            if let Some(wide) = css_wide_keyword(word) {
                return Some(wide);
            }
            if word.eq_ignore_ascii_case("auto")
                && (sizing
                    || (signed
                        && !matches!(
                            property,
                            PropertyId::OutlineOffset
                                | PropertyId::LetterSpacing
                                | PropertyId::WordSpacing
                        )))
            {
                return Some(PropertyValue::Length(Length::Auto));
            }
            if word.eq_ignore_ascii_case("normal")
                && matches!(
                    property,
                    PropertyId::LetterSpacing | PropertyId::WordSpacing | PropertyId::Gap
                )
            {
                return Some(PropertyValue::Length(Length::px(0.0)));
            }
            sizing
                .then(|| layout_keyword::sizing_from_name(word))
                .flatten()
                .map(PropertyValue::Keyword)
        }
        ParsedValue::Number(number) if *number == 0.0 => {
            Some(PropertyValue::Length(Length::px(0.0)))
        }
        ParsedValue::Dimension { value, .. } | ParsedValue::Percentage(value)
            if !signed && *value < 0.0 =>
        {
            None
        }
        _ => match lower_value(values, interner)? {
            value @ (PropertyValue::Length(_) | PropertyValue::Calc(_)) => Some(value),
            _ => None,
        },
    }
}

#[doc = include_str!("../../docs/api/compiler/property_compiler.doc/lower_painter.md")]
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

#[doc = include_str!("../../docs/api/compiler/property_compiler.doc/lower_z_index.md")]
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

/// Picks the unsupported-feature category to report when a value cannot be
/// lowered, based on the shape of its first component.
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

    /// Collects the `(PropertyId, PropertyValue)` pairs of a compiled shorthand's
    /// standard declarations, in emission order.
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
    fn a_property_name_outside_ascii_is_compiled_rather_than_crashed_on() {
        // Authored CSS reaches this compiler unfiltered, from a stylesheet and
        // from a `style="…"` attribute alike, and css-syntax-3 §4.2 admits
        // every code point at or above U+0080 in an identifier. `-a-bcdeé`
        // holds a character across the byte where `-obscur-` ends.
        let (result, _) = compile("-a-bcdeé: red");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected it to be passed over");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::VendorProperty);

        // And one in no namespace at all is an unknown property, not a fault
        // in the compiler.
        let (result, _) = compile("cölor: red");
        let CompiledDeclaration::Unsupported(feature) = result else {
            panic!("expected unsupported");
        };
        assert_eq!(feature.kind(), UnsupportedFeatureKind::Property);
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
    fn a_custom_property_keeps_the_text_it_was_declared_with() {
        let (result, _) = compile("--brand: #fff");
        let CompiledDeclaration::Custom(custom) = result else {
            panic!("expected custom property");
        };
        // Named, not interned: the sheet's interner numbers from zero and every
        // sheet has its own, so a sequential symbol would name a different
        // property in the next sheet compiled.
        assert_eq!(custom.name(), InternedSymbol::named("--brand"));
        // TEXT, not a lowered value. There is no grammar here to lower it
        // against — `#fff` is a colour only once `color` (or any other colour
        // longhand) reads it — and lowering it anyway kept only the first
        // component of everything longer than one.
        assert_eq!(custom.value(), &PropertyValue::RawTokens("#fff".into()));
    }

    #[test]
    fn a_custom_property_keeps_every_component_not_just_the_first() {
        // Audit 2-F, at the seam it lived on. `Inter, sans-serif` used to
        // compile to the single keyword `Inter` — interned, at that, so the
        // symbol named nothing outside this compiler — and `0 2px 4px #000` to
        // the number `0`.
        for (text, expected) in [
            ("--font-body: Inter, sans-serif", "Inter, sans-serif"),
            ("--shadow: 0 2px 4px #000", "0 2px 4px #000"),
            ("--font-body: \"Inter\"", "\"Inter\""),
            ("--ratio: 1.5", "1.5"),
        ] {
            let (result, _) = compile(text);
            let CompiledDeclaration::Custom(custom) = result else {
                panic!("expected a custom property for {text}");
            };
            assert_eq!(
                custom.value(),
                &PropertyValue::RawTokens(expected.into()),
                "{text}"
            );
        }
    }

    #[test]
    fn a_custom_property_that_is_one_var_stays_a_reference() {
        // The one value that is not text: a custom property defined in terms of
        // another is a chain the resolver follows, and text would leave a name
        // nothing resolves.
        let (result, _) = compile("--brand: var(--accent)");
        let CompiledDeclaration::Custom(custom) = result else {
            panic!("expected custom property");
        };
        assert_eq!(
            custom.value(),
            &PropertyValue::CustomRef(InternedSymbol::named("--accent"))
        );
    }

    #[test]
    fn two_compilers_name_the_same_custom_property_the_same_way() {
        // The trap this guards: one `StylesheetCompiler` per source. If the name
        // came from the interner, the FIRST custom property of every sheet would
        // be symbol 0 — the theme's `--brand` and the widget sheet's `--radius`
        // would be the same property, and each would answer the other's `var()`.
        let (first, _) = compile("--brand: #fff");
        let (second, _) = compile("--radius: 4px");
        let (CompiledDeclaration::Custom(brand), CompiledDeclaration::Custom(radius)) =
            (first, second)
        else {
            panic!("expected custom properties");
        };
        assert_ne!(brand.name(), radius.name());
        let (again, _) = compile("--brand: #000");
        let CompiledDeclaration::Custom(again) = again else {
            panic!("expected a custom property");
        };
        assert_eq!(brand.name(), again.name(), "one name, one symbol");
    }

    #[test]
    fn a_var_reference_names_the_property_the_declaration_named() {
        // The two halves are compiled by different code paths (and, for a
        // `style` attribute, by different compiler instances); if they disagreed
        // on the symbol, every `var()` would resolve to nothing.
        let (declaration, _) = compile("--brand: #fff");
        let (reference, _) = standard("color: var(--brand)");
        let CompiledDeclaration::Custom(declaration) = declaration else {
            panic!("expected a custom property");
        };
        assert_eq!(
            reference.value(),
            &PropertyValue::CustomRef(declaration.name())
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
    fn a_value_that_is_one_var_reference_survives_every_grammar() {
        // The consumer audit's 2-C. A `var()` compiled fine as the whole value
        // of a plainly-lowered longhand and was thrown away everywhere else: by
        // a shorthand expander that split the value before lowering it, and by
        // a longhand that parsed its grammar eagerly. Thirty-two declarations
        // in one application, among them EVERY hover and press rule, each of
        // which was `background: var(…)` and nothing else.
        let re = |name: &str| PropertyValue::CustomRef(InternedSymbol::named(name));
        let cases: Vec<(&str, Vec<(PropertyId, PropertyValue)>)> = vec![
            (
                "background: var(--k-face)",
                vec![
                    (PropertyId::BackgroundImage, PropertyValue::Initial),
                    (PropertyId::BackgroundColor, re("--k-face")),
                    (PropertyId::BackgroundSize, PropertyValue::Initial),
                    (PropertyId::BackgroundPosition, PropertyValue::Initial),
                    (PropertyId::BackgroundRepeat, PropertyValue::Initial),
                    (PropertyId::BackgroundBlendMode, PropertyValue::Initial),
                    (PropertyId::BackgroundClip, PropertyValue::Initial),
                    (PropertyId::BackgroundOrigin, PropertyValue::Initial),
                    (PropertyId::BackgroundAttachment, PropertyValue::Initial),
                ],
            ),
            (
                "border-color: var(--k-line)",
                vec![
                    (PropertyId::BorderTopColor, re("--k-line")),
                    (PropertyId::BorderRightColor, re("--k-line")),
                    (PropertyId::BorderBottomColor, re("--k-line")),
                    (PropertyId::BorderLeftColor, re("--k-line")),
                ],
            ),
            (
                "border-radius: var(--k-radius)",
                vec![
                    (PropertyId::BorderTopLeftRadius, re("--k-radius")),
                    (PropertyId::BorderTopRightRadius, re("--k-radius")),
                    (PropertyId::BorderBottomRightRadius, re("--k-radius")),
                    (PropertyId::BorderBottomLeftRadius, re("--k-radius")),
                ],
            ),
            (
                "padding: var(--k-pad)",
                vec![
                    (PropertyId::PaddingTop, re("--k-pad")),
                    (PropertyId::PaddingRight, re("--k-pad")),
                    (PropertyId::PaddingBottom, re("--k-pad")),
                    (PropertyId::PaddingLeft, re("--k-pad")),
                ],
            ),
            (
                "box-shadow: var(--k-shadow)",
                vec![(PropertyId::BoxShadow, re("--k-shadow"))],
            ),
            (
                "font-family: var(--font-body)",
                vec![(PropertyId::FontFamily, re("--font-body"))],
            ),
            (
                "transform: var(--k-lift)",
                vec![(PropertyId::Transform, re("--k-lift"))],
            ),
            (
                "place-items: var(--k-place)",
                vec![
                    (PropertyId::AlignItems, re("--k-place")),
                    (PropertyId::JustifyItems, re("--k-place")),
                ],
            ),
        ];
        for (text, expected) in cases {
            let decls = compile_all(text);
            assert!(
                decls
                    .iter()
                    .all(|d| matches!(d, CompiledDeclaration::Standard(_))),
                "{text} compiled to {decls:?}"
            );
            assert_eq!(pairs(&decls), expected, "{text}");
        }
    }

    #[test]
    fn compile_all_wraps_a_single_longhand() {
        let decls = compile_all("width: 10px");
        assert_eq!(decls.len(), 1);
        assert_eq!(standard_of(&decls[0]).property(), PropertyId::Width);
    }
}
