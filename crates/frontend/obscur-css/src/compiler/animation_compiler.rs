#![doc = include_str!("animation_compiler.doc/_module.md")]

use crate::parser::{ParsedDeclaration, ParsedValue};

use obscur_identity::InternedSymbol;
use obscur_motion::timeline::{FillMode, PlaybackDirection, RepeatCount};

use super::property_compiler::narrow_f32;
use super::stylesheet_compiler::CssSymbolInterner;

#[doc = include_str!("animation_compiler.doc/AnimationRef.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationRef {
    #[doc = include_str!("animation_compiler.doc/name.md")]
    pub name: InternedSymbol,
    #[doc = include_str!("animation_compiler.doc/duration_secs.md")]
    pub duration_secs: f32,
    #[doc = include_str!("animation_compiler.doc/delay_secs.md")]
    pub delay_secs: f32,
    #[doc = include_str!("animation_compiler.doc/repeat.md")]
    pub repeat: RepeatCount,
    #[doc = include_str!("animation_compiler.doc/direction.md")]
    pub direction: PlaybackDirection,
    #[doc = include_str!("animation_compiler.doc/alternates.md")]
    pub alternates: bool,
    #[doc = include_str!("animation_compiler.doc/fill.md")]
    pub fill: FillMode,
}

impl AnimationRef {
    #[doc = include_str!("animation_compiler.doc/new.md")]
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

#[doc = include_str!("animation_compiler.doc/Playback.md")]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Playback {
    #[doc = include_str!("animation_compiler.doc/delay_secs_2.md")]
    delay_secs: f32,
    #[doc = include_str!("animation_compiler.doc/repeat_2.md")]
    repeat: RepeatCount,
    #[doc = include_str!("animation_compiler.doc/direction_2.md")]
    direction: PlaybackDirection,
    #[doc = include_str!("animation_compiler.doc/alternates_2.md")]
    alternates: bool,
    #[doc = include_str!("animation_compiler.doc/fill_2.md")]
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
    #[doc = include_str!("animation_compiler.doc/apply_to.md")]
    fn apply_to(self, mut reference: AnimationRef) -> AnimationRef {
        reference.delay_secs = self.delay_secs;
        reference.repeat = self.repeat;
        reference.direction = self.direction;
        reference.alternates = self.alternates;
        reference.fill = self.fill;
        reference
    }
}

#[doc = include_str!("animation_compiler.doc/direction_from_keyword.md")]
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

#[doc = include_str!("animation_compiler.doc/repeat_from_value.md")]
fn repeat_from_value(value: &ParsedValue) -> Option<RepeatCount> {
    match value {
        ParsedValue::Keyword(keyword) if keyword.eq_ignore_ascii_case("infinite") => {
            Some(RepeatCount::Infinite)
        }
        ParsedValue::Number(n) if *n >= 0.0 => Some(RepeatCount::Finite(whole_iterations(*n))),
        _ => None,
    }
}

#[doc = include_str!("animation_compiler.doc/AnimationCompiler.md")]
#[derive(Debug, Clone, Copy, Default)]
pub struct AnimationCompiler;

impl AnimationCompiler {
    #[doc = include_str!("animation_compiler.doc/new_2.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("animation_compiler.doc/compile.md")]
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

#[doc = include_str!("animation_compiler.doc/whole_iterations.md")]
// The value is rounded to an integer and clamped to `0..=u32::MAX` first, so
// neither truncation nor sign loss can occur.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn whole_iterations(count: f64) -> u32 {
    let clamped = count.round().clamp(0.0, f64::from(u32::MAX));
    clamped as u32
}

#[doc = include_str!("animation_compiler.doc/split_layers.md")]
fn split_layers(values: &[ParsedValue]) -> Vec<&[ParsedValue]> {
    values
        .split(|value| matches!(value, ParsedValue::Comma))
        .filter(|layer| !layer.is_empty())
        .collect()
}

#[doc = include_str!("animation_compiler.doc/compile_shorthand_layer.md")]
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

#[doc = include_str!("animation_compiler.doc/first_time_secs.md")]
fn first_time_secs(values: &[ParsedValue]) -> Option<f32> {
    values.iter().find_map(|value| match value {
        ParsedValue::Dimension { value, unit } => time_secs(*value, unit),
        _ => None,
    })
}

#[doc = include_str!("animation_compiler.doc/time_secs.md")]
fn time_secs(value: f64, unit: &str) -> Option<f32> {
    if unit.eq_ignore_ascii_case("s") {
        Some(narrow_f32(value))
    } else if unit.eq_ignore_ascii_case("ms") {
        Some(narrow_f32(value) / 1000.0)
    } else {
        None
    }
}

#[doc = include_str!("animation_compiler.doc/is_reserved_keyword.md")]
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
