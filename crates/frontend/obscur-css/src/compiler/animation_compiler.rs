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
//! longhands (`animation-name: pulse; animation-duration: 1s`) are recognised,
//! as are `animation-delay`, `animation-iteration-count`, `animation-direction`
//! and `animation-fill-mode` — each lowered onto the [`AnimationRef`] the
//! runtime turns into a [`Timeline`](obscur_motion::timeline::Timeline).
//! `animation-play-state` and `animation-timing-function` are still ignored.

use crate::parser::{ParsedDeclaration, ParsedValue};

use obscur_identity::InternedSymbol;
use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};

use super::property_compiler::narrow_f32;
use super::stylesheet_compiler::CssSymbolInterner;

/// A single `@keyframes` reference declared by a style rule: the interned
/// animation name, its duration, and the playback controls the `animation-*`
/// longhands set (css-animations-1 §3.3–§3.9).
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
    /// `animation-delay`, in seconds. Negative values seek into the playback
    /// (css-animations-1 §3.8).
    pub delay_secs: f32,
    /// `animation-iteration-count`, as a repeat count.
    pub repeat: RepeatCount,
    /// The direction iteration 1 plays in — `reverse`/`alternate-reverse` start
    /// reversed (css-animations-1 §3.6).
    pub direction: PlaybackDirection,
    /// Whether each iteration flips direction (`alternate`/`alternate-reverse`).
    pub alternates: bool,
    /// `animation-fill-mode` (css-animations-1 §3.9).
    pub fill: FillMode,
}

impl AnimationRef {
    /// A reference to `name` with every playback control at its CSS initial
    /// value: no delay, one iteration, `normal` direction, `none` fill.
    #[must_use]
    fn new(name: InternedSymbol, duration_secs: f32) -> Self {
        Self {
            name,
            duration_secs,
            delay_secs: 0.0,
            repeat: RepeatCount::Finite(1),
            direction: PlaybackDirection::Forward,
            alternates: false,
            fill: FillMode::None,
        }
    }
}

/// The playback controls an `animation` layer or the `animation-*` longhands
/// specify, independent of which `@keyframes` they apply to.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Playback {
    /// `animation-delay`, in seconds.
    delay_secs: f32,
    /// `animation-iteration-count`.
    repeat: RepeatCount,
    /// The direction iteration 1 plays in.
    direction: PlaybackDirection,
    /// Whether iterations alternate direction.
    alternates: bool,
    /// `animation-fill-mode`.
    fill: FillMode,
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            delay_secs: 0.0,
            repeat: RepeatCount::Finite(1),
            direction: PlaybackDirection::Forward,
            alternates: false,
            fill: FillMode::None,
        }
    }
}

impl Playback {
    /// Stamps these controls onto `reference`.
    fn apply_to(self, mut reference: AnimationRef) -> AnimationRef {
        reference.delay_secs = self.delay_secs;
        reference.repeat = self.repeat;
        reference.direction = self.direction;
        reference.alternates = self.alternates;
        reference.fill = self.fill;
        reference
    }
}

/// Maps a `<single-animation-direction>` keyword to the direction iteration 1
/// plays in plus whether iterations alternate (css-animations-1 §3.6).
///
/// `alternate` starts normal and flips; `alternate-reverse` starts reversed and
/// flips. Because iteration 1 is the first one played, this is exactly the
/// spec's one-based parity: "iterations start counting from 1".
fn direction_from_keyword(keyword: &str) -> Option<(PlaybackDirection, bool)> {
    if keyword.eq_ignore_ascii_case("normal") {
        Some((PlaybackDirection::Forward, false))
    } else if keyword.eq_ignore_ascii_case("reverse") {
        Some((PlaybackDirection::Reverse, false))
    } else if keyword.eq_ignore_ascii_case("alternate") {
        Some((PlaybackDirection::Forward, true))
    } else if keyword.eq_ignore_ascii_case("alternate-reverse") {
        Some((PlaybackDirection::Reverse, true))
    } else {
        None
    }
}

/// Maps an `<single-animation-iteration-count>` component value to a
/// [`RepeatCount`]: `infinite`, or a non-negative number rounded to whole
/// iterations. Fractional counts are not modelled and round to the nearest
/// whole iteration.
fn repeat_from_value(value: &ParsedValue) -> Option<RepeatCount> {
    match value {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("infinite") => {
            Some(RepeatCount::Infinite)
        }
        ParsedValue::Number(n) if *n >= 0.0 => Some(RepeatCount::Finite(whole_iterations(*n))),
        _ => None,
    }
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
    /// duration, delay, iteration count, direction and fill mode); bare
    /// `animation-name` idents each contribute one ref carrying the
    /// `animation-duration` longhand's value (or `0` when absent) and the
    /// playback longhands. An `animation-name: none` layer is skipped.
    ///
    /// A playback longhand declared **after** the `animation` shorthand
    /// overrides it on every shorthand-derived ref (`animation: a 1s;
    /// animation-direction: alternate`); one declared *before* it is reset by
    /// the shorthand, as the cascade requires. Per-layer longhand lists are not
    /// modelled: one longhand value applies to all of a rule's refs.
    #[must_use]
    pub fn compile(
        &self,
        decls: &[ParsedDeclaration],
        interner: &mut CssSymbolInterner,
    ) -> Vec<AnimationRef> {
        let mut shorthand: Vec<AnimationRef> = Vec::new();
        let mut long_names: Vec<InternedSymbol> = Vec::new();
        let mut long_duration: f32 = 0.0;
        // Playback longhands, and the declaration index each was authored at (an
        // unauthored one must not clobber what the shorthand set, and one that
        // precedes the shorthand is reset by it).
        let mut long_playback = Playback::default();
        let mut long_delay_at: Option<usize> = None;
        let mut long_repeat_at: Option<usize> = None;
        let mut long_direction_at: Option<usize> = None;
        let mut long_fill_at: Option<usize> = None;
        let mut shorthand_at: Option<usize> = None;

        for (index, decl) in decls.iter().enumerate() {
            let name = decl.name.to_ascii_lowercase();
            match name.as_str() {
                "animation" => {
                    shorthand_at = Some(index);
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
                "animation-delay" => {
                    if let Some(secs) = first_time_secs(&decl.value) {
                        long_playback.delay_secs = secs;
                        long_delay_at = Some(index);
                    }
                }
                "animation-iteration-count" => {
                    if let Some(repeat) = decl.value.iter().find_map(repeat_from_value) {
                        long_playback.repeat = repeat;
                        long_repeat_at = Some(index);
                    }
                }
                "animation-direction" => {
                    if let Some((direction, alternates)) =
                        decl.value.iter().find_map(|value| match value {
                            ParsedValue::Keyword(keyword) => direction_from_keyword(keyword),
                            _ => None,
                        })
                    {
                        long_playback.direction = direction;
                        long_playback.alternates = alternates;
                        long_direction_at = Some(index);
                    }
                }
                "animation-fill-mode" => {
                    if let Some(fill) = decl.value.iter().find_map(|value| match value {
                        ParsedValue::Keyword(keyword) => FillMode::from_keyword(keyword),
                        _ => None,
                    }) {
                        long_playback.fill = fill;
                        long_fill_at = Some(index);
                    }
                }
                _ => {}
            }
        }

        let mut out = shorthand;
        // Overlay only the longhands that were authored *after* the shorthand
        // they would override.
        let wins = |at: Option<usize>| match (at, shorthand_at) {
            (Some(longhand), Some(short)) => longhand > short,
            (Some(_), None) => true,
            (None, _) => false,
        };
        for reference in &mut out {
            if wins(long_delay_at) {
                reference.delay_secs = long_playback.delay_secs;
            }
            if wins(long_repeat_at) {
                reference.repeat = long_playback.repeat;
            }
            if wins(long_direction_at) {
                reference.direction = long_playback.direction;
                reference.alternates = long_playback.alternates;
            }
            if wins(long_fill_at) {
                reference.fill = long_playback.fill;
            }
        }
        for name in long_names {
            out.push(long_playback.apply_to(AnimationRef::new(name, long_duration)));
        }
        out
    }
}

/// Rounds a non-negative iteration count to whole iterations, clamped into
/// `u32`. A count above `u32::MAX` saturates (it is indistinguishable from
/// `infinite` in practice, but is still finite).
// The value is rounded to an integer and clamped to `0..=u32::MAX` first, so
// neither truncation nor sign loss can occur.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn whole_iterations(count: f64) -> u32 {
    let clamped = count.round().clamp(0.0, f64::from(u32::MAX));
    clamped as u32
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
/// is taken as the animation name. Per css-animations-1 §3.10, "the first value
/// in each `<single-animation>` that can be parsed as a `<time>` is assigned to
/// animation-duration, and the second value … is assigned to animation-delay";
/// the first `<number>`/`infinite` is the iteration count, and the direction and
/// fill-mode keywords are recognised by name.
fn compile_shorthand_layer(
    layer: &[ParsedValue],
    interner: &mut CssSymbolInterner,
) -> Option<AnimationRef> {
    let mut name: Option<InternedSymbol> = None;
    let mut times: Vec<f32> = Vec::new();
    let mut playback = Playback::default();
    let mut repeat_seen = false;
    for component in layer {
        match component {
            ParsedValue::Keyword(keyword) => {
                if let Some((direction, alternates)) = direction_from_keyword(keyword) {
                    // `normal` is both the initial value and a real keyword; either
                    // way recording it is correct.
                    playback.direction = direction;
                    playback.alternates = alternates;
                } else if keyword.eq_ignore_ascii_case("infinite") {
                    playback.repeat = RepeatCount::Infinite;
                    repeat_seen = true;
                } else if let Some(fill) = FillMode::from_keyword(keyword) {
                    // `none` reaches here only as a fill mode: the name slot never
                    // takes a reserved keyword.
                    playback.fill = fill;
                } else if !is_reserved_keyword(keyword) && name.is_none() {
                    name = Some(interner.intern(keyword));
                }
            }
            ParsedValue::Dimension { value, unit } => {
                if let Some(secs) = time_secs(*value, unit) {
                    times.push(secs);
                }
            }
            ParsedValue::Number(_) if !repeat_seen => {
                // A negative count is invalid and yields nothing.
                if let Some(repeat) = repeat_from_value(component) {
                    playback.repeat = repeat;
                    repeat_seen = true;
                }
            }
            _ => {}
        }
    }
    playback.delay_secs = times.get(1).copied().unwrap_or(0.0);
    let duration = times.first().copied().unwrap_or(0.0);
    name.map(|name| playback.apply_to(AnimationRef::new(name, duration)))
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
    fn shorthand_second_time_is_the_delay() {
        // css-animations-1 §3.10: the second <time> is animation-delay.
        let (refs, _) = compile("animation: a 1s 250ms");
        assert_eq!(refs[0].duration_secs, 1.0);
        assert_eq!(refs[0].delay_secs, 0.25);
    }

    #[test]
    fn shorthand_negative_delay_survives() {
        let (refs, _) = compile("animation: a 2s -0.5s");
        assert_eq!(refs[0].delay_secs, -0.5);
    }

    #[test]
    fn shorthand_iteration_count_and_infinite() {
        assert_eq!(
            compile("animation: a 1s 3").0[0].repeat,
            RepeatCount::Finite(3)
        );
        assert_eq!(
            compile("animation: a 1s infinite").0[0].repeat,
            RepeatCount::Infinite
        );
        assert_eq!(
            compile("animation: a 1s").0[0].repeat,
            RepeatCount::Finite(1),
            "the initial iteration count is 1"
        );
    }

    #[test]
    fn shorthand_directions_map_to_start_direction_and_alternation() {
        // css-animations-1 §3.6, with iterations counting from 1: `alternate`
        // plays iteration 1 normally, `alternate-reverse` plays it reversed.
        let normal = compile("animation: a 1s normal").0[0];
        assert_eq!(normal.direction, PlaybackDirection::Forward);
        assert!(!normal.alternates);
        let reverse = compile("animation: a 1s reverse").0[0];
        assert_eq!(reverse.direction, PlaybackDirection::Reverse);
        assert!(!reverse.alternates);
        let alternate = compile("animation: a 1s alternate").0[0];
        assert_eq!(alternate.direction, PlaybackDirection::Forward);
        assert!(alternate.alternates);
        let alt_rev = compile("animation: a 1s alternate-reverse").0[0];
        assert_eq!(alt_rev.direction, PlaybackDirection::Reverse);
        assert!(alt_rev.alternates);
    }

    #[test]
    fn shorthand_fill_modes_map() {
        assert_eq!(compile("animation: a 1s").0[0].fill, FillMode::None);
        assert_eq!(
            compile("animation: a 1s forwards").0[0].fill,
            FillMode::Forwards
        );
        assert_eq!(
            compile("animation: a 1s backwards").0[0].fill,
            FillMode::Backwards
        );
        assert_eq!(compile("animation: a 1s both").0[0].fill, FillMode::Both);
    }

    #[test]
    fn longhands_carry_every_playback_control() {
        let (refs, _) = compile(
            "animation-name: a; animation-duration: 1s; animation-delay: 2s; \
             animation-iteration-count: 4; animation-direction: alternate-reverse; \
             animation-fill-mode: both",
        );
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].delay_secs, 2.0);
        assert_eq!(refs[0].repeat, RepeatCount::Finite(4));
        assert_eq!(refs[0].direction, PlaybackDirection::Reverse);
        assert!(refs[0].alternates);
        assert_eq!(refs[0].fill, FillMode::Both);
    }

    #[test]
    fn a_longhand_overrides_the_shorthand_it_follows() {
        let (refs, _) = compile("animation: a 1s; animation-iteration-count: infinite");
        assert_eq!(refs[0].repeat, RepeatCount::Infinite);
    }

    #[test]
    fn a_longhand_the_shorthand_follows_is_reset_by_it() {
        // The shorthand resets every longhand it can set, so the earlier
        // iteration count does not survive it.
        let (refs, _) = compile("animation-iteration-count: infinite; animation: a 1s");
        assert_eq!(refs[0].repeat, RepeatCount::Finite(1));
    }

    #[test]
    fn an_unauthored_longhand_does_not_clobber_the_shorthand() {
        let (refs, _) = compile("animation: a 1s 3 alternate backwards");
        assert_eq!(refs[0].repeat, RepeatCount::Finite(3));
        assert!(refs[0].alternates);
        assert_eq!(refs[0].fill, FillMode::Backwards);
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
