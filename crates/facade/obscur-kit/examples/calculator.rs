#![doc = include_str!("calculator.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::App;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{EVENT_LONG_PRESS, View, ViewEventBinding, ViewNode};
use obscur_kit::widgets::button::Button;

#[doc = include_str!("calculator.doc/TAPE.md")]
const TAPE: InternedSymbol = InternedSymbol::named("tape");

#[doc = include_str!("calculator.doc/TAPE_ROW.md")]
const TAPE_ROW: f32 = 28.0;

#[doc = include_str!("calculator.doc/CMD_CLEAR_ALL.md")]
const CMD_CLEAR_ALL: InternedSymbol = InternedSymbol::named("clearAll");

#[doc = include_str!("calculator.doc/CSS.md")]
const CSS: &str = "\
.safe       { background-color: rgba(13, 16, 23, 0.62); width: 100%; height: 100%; \
              display: flex; flex-direction: column; \
              justify-content: center; align-items: center; \
              padding-top: env(safe-area-inset-top); \
              padding-bottom: env(safe-area-inset-bottom); \
              padding-left: env(safe-area-inset-left); \
              padding-right: env(safe-area-inset-right); }\n\
.app        { flex: 1; max-width: 460px; max-height: 780px; \
              display: flex; flex-direction: column; gap: 10px; padding: 12px; }\n\
.tape       { flex: 3; background-color: rgba(20, 25, 38, 0.55); border-radius: min(3vmin, 14px); \
              display: flex; flex-direction: column; overflow: scroll; }\n\
.tape-row   { height: 30px; font-size: min(3.4vmin, 18px); white-space: nowrap; \
              display: flex; flex-direction: row; \
              justify-content: flex-end; align-items: center; \
              padding-left: 14px; padding-right: 14px; overflow: hidden; }\n\
.tape-row:hover { background-color: rgba(27, 34, 51, 0.75); }\n\
.tape-text  { color: #7c89ad; font-size: min(3.4vmin, 18px); white-space: nowrap; \
              overflow: hidden; text-overflow: ellipsis; }\n\
.tape-empty { color: #3f4a66; font-size: min(3.4vmin, 18px); }\n\
.display    { flex: 2; background-color: rgba(20, 25, 38, 0.55); border-radius: min(3vmin, 14px); \
              overflow: hidden; \
              display: flex; flex-direction: column; justify-content: center; \
              padding-left: 18px; padding-right: 18px; }\n\
.pending-row{ overflow: hidden; font-size: min(4vmin, 22px); \
              display: flex; flex-direction: row; \
              justify-content: flex-end; align-items: center; }\n\
.value-row  { overflow: hidden; font-size: min(9vmin, 52px); \
              display: flex; flex-direction: row; \
              justify-content: flex-end; align-items: center; }\n\
.pending    { color: #6b7799; font-size: min(4vmin, 22px); }\n\
.value      { color: #f2f5ff; font-size: min(9vmin, 52px); }\n\
.pad        { flex: 8; display: flex; flex-direction: column; gap: 10px; }\n\
.krow       { flex: 1; display: flex; flex-direction: row; gap: 10px; }\n\
.key        { flex: 1; background-color: rgba(28, 34, 51, 0.72); border-radius: min(3vmin, 14px); \
              padding: 0; border: 0; overflow: hidden; \
              color: #eef2ff; font-size: min(6vmin, 34px); \
              display: flex; justify-content: center; align-items: center; }\n\
.key:hover  { background-color: rgba(38, 46, 69, 0.85); }\n\
.key:active { background-color: rgba(67, 80, 122, 0.95); }\n\
.wide       { flex: 2; }\n\
.fn         { background-color: rgba(22, 27, 41, 0.66); color: #b9c4e0; \
              font-size: min(4.6vmin, 26px); }\n\
.fn:active  { background-color: rgba(51, 61, 92, 0.95); }\n\
.op         { background-color: rgba(35, 44, 73, 0.78); color: #cfe0ff; }\n\
.op:active  { background-color: rgba(70, 89, 143, 0.95); }\n\
.eq         { background-color: rgba(52, 98, 196, 0.88); color: #ffffff; }\n\
.eq:active  { background-color: rgba(91, 134, 232, 0.97); }\n\
";

#[doc = include_str!("calculator.doc/Key.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Key {
    Digit(u8),
    Dot,
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    Back,
    Sign,
    Percent,
}

impl Key {
    #[doc = include_str!("calculator.doc/symbol.md")]
    const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "−",
            Self::Mul => "×",
            Self::Div => "÷",
            _ => "",
        }
    }

    #[doc = include_str!("calculator.doc/apply.md")]
    fn apply(self, a: f64, b: f64) -> f64 {
        match self {
            Self::Add => a + b,
            Self::Sub => a - b,
            Self::Mul => a * b,
            Self::Div => a / b,
            _ => b,
        }
    }
}

#[doc = include_str!("calculator.doc/KEYS.md")]
const KEYS: &[(&str, &str, Key, &str)] = &[
    ("back", "⌫", Key::Back, "fn"),
    ("sign", "±", Key::Sign, "fn"),
    ("pct", "%", Key::Percent, "fn"),
    ("div", "÷", Key::Div, "op"),
    ("d7", "7", Key::Digit(7), ""),
    ("d8", "8", Key::Digit(8), ""),
    ("d9", "9", Key::Digit(9), ""),
    ("mul", "×", Key::Mul, "op"),
    ("d4", "4", Key::Digit(4), ""),
    ("d5", "5", Key::Digit(5), ""),
    ("d6", "6", Key::Digit(6), ""),
    ("sub", "−", Key::Sub, "op"),
    ("d1", "1", Key::Digit(1), ""),
    ("d2", "2", Key::Digit(2), ""),
    ("d3", "3", Key::Digit(3), ""),
    ("add", "+", Key::Add, "op"),
    ("d0", "0", Key::Digit(0), "wide"),
    ("dot", ".", Key::Dot, ""),
    ("eq", "=", Key::Equals, "eq"),
];

#[doc = include_str!("calculator.doc/ROWS.md")]
const ROWS: &[&[usize]] = &[
    &[0, 1, 2, 3],
    &[4, 5, 6, 7],
    &[8, 9, 10, 11],
    &[12, 13, 14, 15],
    &[16, 17, 18],
];

#[doc = include_str!("calculator.doc/MAX_DIGITS.md")]
const MAX_DIGITS: usize = 12;

#[doc = include_str!("calculator.doc/Calc.md")]
struct Calc {
    #[doc = include_str!("calculator.doc/entry.md")]
    entry: String,
    #[doc = include_str!("calculator.doc/acc.md")]
    acc: Option<f64>,
    #[doc = include_str!("calculator.doc/op.md")]
    op: Option<Key>,
    #[doc = include_str!("calculator.doc/fresh.md")]
    fresh: bool,
    #[doc = include_str!("calculator.doc/tape.md")]
    tape: Vec<String>,
}

impl Calc {
    fn new() -> Self {
        Self {
            entry: "0".to_owned(),
            acc: None,
            op: None,
            fresh: true,
            tape: Vec::new(),
        }
    }

    #[doc = include_str!("calculator.doc/value.md")]
    fn value(&self) -> f64 {
        self.entry.parse().unwrap_or(0.0)
    }

    #[doc = include_str!("calculator.doc/format.md")]
    fn format(value: f64) -> String {
        if value.is_finite() && value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{value:.0}")
        } else {
            let text = format!("{value:.6}");
            let trimmed = text.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_owned()
        }
    }

    #[doc = include_str!("calculator.doc/pending.md")]
    fn pending(&self) -> String {
        match (self.acc, self.op) {
            (Some(acc), Some(op)) => format!("{} {}", Self::format(acc), op.symbol()),
            _ => String::new(),
        }
    }

    #[doc = include_str!("calculator.doc/press.md")]
    fn press(&mut self, key: Key) {
        match key {
            Key::Digit(d) => {
                if self.fresh {
                    self.entry.clear();
                    self.fresh = false;
                }
                if self.entry == "0" {
                    self.entry.clear();
                }
                if self.entry.len() < MAX_DIGITS {
                    self.entry.push(char::from(b'0' + d));
                }
                if self.entry.is_empty() {
                    self.entry.push('0');
                }
            }
            Key::Dot => {
                if self.fresh {
                    "0".clone_into(&mut self.entry);
                    self.fresh = false;
                }
                if !self.entry.contains('.') {
                    self.entry.push('.');
                }
            }
            Key::Add | Key::Sub | Key::Mul | Key::Div => {
                self.fold();
                self.op = Some(key);
                self.fresh = true;
            }
            Key::Equals => {
                if let (Some(acc), Some(op)) = (self.acc, self.op) {
                    let rhs = self.value();
                    let result = op.apply(acc, rhs);
                    self.tape.insert(
                        0,
                        format!(
                            "{} {} {} = {}",
                            Self::format(acc),
                            op.symbol(),
                            Self::format(rhs),
                            Self::format(result)
                        ),
                    );
                    self.entry = Self::format(result);
                }
                self.acc = None;
                self.op = None;
                self.fresh = true;
            }
            Key::Back => {
                if self.fresh {
                    "0".clone_into(&mut self.entry);
                    self.fresh = false;
                } else {
                    self.entry.pop();
                }
                if self.entry.is_empty() || self.entry == "-" {
                    "0".clone_into(&mut self.entry);
                }
            }
            Key::Sign => {
                if self.entry.starts_with('-') {
                    self.entry.remove(0);
                } else if self.entry != "0" {
                    self.entry.insert(0, '-');
                }
            }
            Key::Percent => {
                self.entry = Self::format(self.value() / 100.0);
                self.fresh = false;
            }
        }
    }

    #[doc = include_str!("calculator.doc/fold.md")]
    fn fold(&mut self) {
        let value = self.value();
        self.acc = Some(match (self.acc, self.op) {
            (Some(acc), Some(op)) if !self.fresh => op.apply(acc, value),
            (Some(acc), _) if self.fresh => acc,
            _ => value,
        });
    }

    #[doc = include_str!("calculator.doc/clear_all.md")]
    fn clear_all(&mut self) {
        "0".clone_into(&mut self.entry);
        self.acc = None;
        self.op = None;
        self.fresh = true;
    }

    #[doc = include_str!("calculator.doc/recall.md")]
    fn recall(&mut self, index: usize) {
        let Some(line) = self.tape.get(index) else {
            return;
        };
        if let Some((_, result)) = line.rsplit_once(" = ") {
            result.clone_into(&mut self.entry);
            self.fresh = false;
        }
    }

    #[doc = include_str!("calculator.doc/forget.md")]
    fn forget(&mut self, index: usize) {
        if index < self.tape.len() {
            self.tape.remove(index);
        }
    }
}

fn main() -> Result<(), obscur_kit::AppError> {
    let state = Rc::new(RefCell::new(Calc::new()));

    let view_state = state.clone();
    let row_state = state.clone();
    let clear_state = state.clone();
    let tap_state = state.clone();
    let hold_state = state.clone();

    let mut app = App::new("obscur-kit — calculator (touch)")
        .app_id("obscur-kit-calculator")
        .with_size(360.0, 620.0)
        .stylesheet(CSS)
        .view(move || build_view(&view_state.borrow()))
        // The tape is a kit-owned virtual list, so the runtime windows it, drags
        // it with the finger and coasts it after a flick — none of which this app
        // implements.
        .virtual_list(TAPE, move |index| tape_row(&row_state.borrow(), index))
        .on_row_click(TAPE, move |index, _clicks| {
            tap_state.borrow_mut().recall(index);
        })
        .on_row_long_press(TAPE, move |index| hold_state.borrow_mut().forget(index))
        .on_command(CMD_CLEAR_ALL, move || clear_state.borrow_mut().clear_all());

    // One handler per key, from the same table the keypad is built from — so a
    // key cannot exist on screen without a handler, or the reverse.
    for &(name, _, key, _) in KEYS {
        let pressed = state.clone();
        app = app.on_command(InternedSymbol::named(name), move || {
            pressed.borrow_mut().press(key);
        });
    }

    app.run()
}

#[doc = include_str!("calculator.doc/build_view.md")]
fn build_view(state: &Calc) -> View {
    let app = Container::new()
        .child(tape(state))
        .child(display(state))
        .child(keypad())
        .into_view_node()
        .class("app");
    View::single(Container::new().child(app).into_view_node().class("safe"))
}

#[doc = include_str!("calculator.doc/keypad.md")]
fn keypad() -> ViewNode {
    Container::new()
        .with_children(ROWS.iter().map(|row| key_row(row)).collect::<Vec<_>>())
        .into_view_node()
        .class("pad")
}

#[doc = include_str!("calculator.doc/tape_2.md")]
fn tape(state: &Calc) -> ViewNode {
    let child = if state.tape.is_empty() {
        Container::new()
            .child(
                Text::new("no history yet")
                    .into_view_node()
                    .class("tape-empty"),
            )
            .into_view_node()
            .class("tape-row")
    } else {
        ViewNode::virtual_list(TAPE, state.tape.len(), TAPE_ROW)
    };
    Container::new().child(child).into_view_node().class("tape")
}

#[doc = include_str!("calculator.doc/tape_row.md")]
fn tape_row(state: &Calc, index: usize) -> ViewNode {
    let text = state.tape.get(index).cloned().unwrap_or_default();
    Container::new()
        .child(Text::new(text).into_view_node().class("tape-text"))
        .into_view_node()
        .class("tape-row")
}

#[doc = include_str!("calculator.doc/display.md")]
fn display(state: &Calc) -> ViewNode {
    let line = |class: &str, row: &str, text: String| {
        Container::new()
            .child(Text::new(text).into_view_node().class(class.to_owned()))
            .into_view_node()
            .class(row.to_owned())
    };
    Container::new()
        .child(line("pending", "pending-row", state.pending()))
        .child(line("value", "value-row", state.entry.clone()))
        .into_view_node()
        .class("display")
}

#[doc = include_str!("calculator.doc/key_row.md")]
fn key_row(row: &[usize]) -> ViewNode {
    Container::new()
        .with_children(row.iter().map(|&i| key(i)).collect::<Vec<_>>())
        .into_view_node()
        .class("krow")
}

#[doc = include_str!("calculator.doc/key.md")]
fn key(index: usize) -> ViewNode {
    let (name, label, key, extra) = KEYS[index];
    let node = Button::new(label, InternedSymbol::named(name))
        .into_view_node()
        .class(if extra.is_empty() {
            "key".to_owned()
        } else {
            format!("key {extra}")
        });
    if key == Key::Back {
        node.with_event(ViewEventBinding::new(EVENT_LONG_PRESS, CMD_CLEAR_ALL))
    } else {
        node
    }
}

#[cfg(test)]
mod tests {
    use super::{Calc, Key};

    /// Presses a run of keys, so a case reads like the sequence a thumb makes.
    fn press(calc: &mut Calc, keys: &[Key]) {
        for &key in keys {
            calc.press(key);
        }
    }

    fn digits(text: &str) -> Vec<Key> {
        text.chars()
            .map(|c| match c {
                '.' => Key::Dot,
                d => Key::Digit(d as u8 - b'0'),
            })
            .collect()
    }

    #[test]
    fn typing_digits_builds_the_entry() {
        let mut c = Calc::new();
        press(&mut c, &digits("120"));
        assert_eq!(c.entry, "120", "the leading 0 was replaced, not kept");
    }

    #[test]
    fn an_entry_is_bounded() {
        // A thumb resting on a key must not grow the string without bound.
        let mut c = Calc::new();
        press(&mut c, &digits("1234567890123456789"));
        assert_eq!(c.entry.len(), super::MAX_DIGITS);
    }

    #[test]
    fn a_sum_computes_and_writes_the_tape() {
        let mut c = Calc::new();
        press(&mut c, &digits("2"));
        press(&mut c, &[Key::Add]);
        press(&mut c, &digits("3"));
        press(&mut c, &[Key::Equals]);

        assert_eq!(c.entry, "5");
        assert_eq!(c.tape, vec!["2 + 3 = 5".to_owned()]);
    }

    /// Chaining folds as you go, so the display shows the running total when the
    /// second operator is pressed — not only at `=`.
    #[test]
    fn chaining_folds_the_pending_operation() {
        let mut c = Calc::new();
        press(&mut c, &digits("2"));
        press(&mut c, &[Key::Add]);
        press(&mut c, &digits("3"));
        press(&mut c, &[Key::Add]);
        assert_eq!(c.acc, Some(5.0), "2 + 3 folded when the second + arrived");
        press(&mut c, &digits("4"));
        press(&mut c, &[Key::Equals]);
        assert_eq!(c.entry, "9");
    }

    #[test]
    fn a_repeated_operator_does_not_fold_twice() {
        // `2 + +` must stay 2, not become 4.
        let mut c = Calc::new();
        press(&mut c, &digits("2"));
        press(&mut c, &[Key::Add, Key::Add, Key::Add]);
        assert_eq!(c.acc, Some(2.0));
    }

    #[test]
    fn equals_with_nothing_pending_is_inert() {
        let mut c = Calc::new();
        press(&mut c, &digits("7"));
        press(&mut c, &[Key::Equals]);
        assert_eq!(c.entry, "7");
        assert!(
            c.tape.is_empty(),
            "nothing was computed, so nothing is recorded"
        );
    }

    #[test]
    fn division_and_decimals_round_trip() {
        let mut c = Calc::new();
        press(&mut c, &digits("1"));
        press(&mut c, &[Key::Div]);
        press(&mut c, &digits("4"));
        press(&mut c, &[Key::Equals]);
        assert_eq!(c.entry, "0.25");
    }

    #[test]
    fn a_dot_cannot_be_doubled() {
        let mut c = Calc::new();
        press(&mut c, &digits("1.5"));
        press(&mut c, &[Key::Dot, Key::Dot]);
        assert_eq!(c.entry, "1.5");
    }

    #[test]
    fn backspace_deletes_one_digit_and_stops_at_zero() {
        let mut c = Calc::new();
        press(&mut c, &digits("12"));
        press(&mut c, &[Key::Back]);
        assert_eq!(c.entry, "1");
        press(&mut c, &[Key::Back, Key::Back]);
        assert_eq!(c.entry, "0", "it bottoms out rather than emptying");
    }

    /// The pair to the test above, and the reason `⌫` binds a long press: held,
    /// it is a different action, not a faster one.
    #[test]
    fn clear_all_forgets_the_pending_operation_too() {
        let mut c = Calc::new();
        press(&mut c, &digits("2"));
        press(&mut c, &[Key::Add]);
        press(&mut c, &digits("3"));
        c.clear_all();
        assert_eq!(c.entry, "0");
        assert_eq!(c.acc, None);
        assert_eq!(c.op, None);
    }

    #[test]
    fn sign_toggles_but_leaves_zero_alone() {
        let mut c = Calc::new();
        press(&mut c, &[Key::Sign]);
        assert_eq!(c.entry, "0", "there is no negative zero to show");
        press(&mut c, &digits("5"));
        press(&mut c, &[Key::Sign]);
        assert_eq!(c.entry, "-5");
        press(&mut c, &[Key::Sign]);
        assert_eq!(c.entry, "5");
    }

    #[test]
    fn percent_scales_the_entry() {
        let mut c = Calc::new();
        press(&mut c, &digits("50"));
        press(&mut c, &[Key::Percent]);
        assert_eq!(c.entry, "0.5");
    }

    #[test]
    fn a_tapped_tape_line_recalls_its_result() {
        let mut c = Calc::new();
        press(&mut c, &digits("6"));
        press(&mut c, &[Key::Mul]);
        press(&mut c, &digits("7"));
        press(&mut c, &[Key::Equals]);
        press(&mut c, &digits("1"));

        c.recall(0);
        assert_eq!(c.entry, "42", "the tape line's result, not its expression");
    }

    #[test]
    fn a_held_tape_line_is_deleted() {
        let mut c = Calc::new();
        for n in ["1", "2"] {
            press(&mut c, &digits(n));
            press(&mut c, &[Key::Add]);
            press(&mut c, &digits("1"));
            press(&mut c, &[Key::Equals]);
        }
        assert_eq!(c.tape.len(), 2);
        c.forget(0);
        assert_eq!(c.tape, vec!["1 + 1 = 2".to_owned()], "the newest went");
        c.forget(9);
        assert_eq!(c.tape.len(), 1, "an index past the end is harmless");
    }

    #[test]
    fn the_pending_line_shows_the_running_left_hand_side() {
        let mut c = Calc::new();
        assert_eq!(c.pending(), "");
        press(&mut c, &digits("8"));
        press(&mut c, &[Key::Sub]);
        assert_eq!(c.pending(), "8 −");
    }

    #[test]
    fn every_key_on_screen_has_a_handler_and_a_place() {
        // The keypad and the handler loop are both built from `KEYS`, so this
        // pins the third thing that has to agree: that `ROWS` lays out every key
        // exactly once.
        let mut placed: Vec<usize> = super::ROWS.iter().flat_map(|r| r.iter().copied()).collect();
        placed.sort_unstable();
        let expected: Vec<usize> = (0..super::KEYS.len()).collect();
        assert_eq!(placed, expected, "every key is laid out exactly once");
    }
}
