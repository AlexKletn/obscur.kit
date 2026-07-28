//! A touch-first calculator — the kit's gesture stack in something you can
//! actually use with a thumb.
//!
//! It exists because a calculator is the smallest honest test of touch input:
//! every interaction is a *tap on a small target*, which is where a pointer stack
//! either works or is exhausting. Everything the kit grew for touch shows up here
//! in the shape an application would really use it:
//!
//! - **Taps with feedback.** Keys are 80×68 (comfortably past a fingertip's ~9mm)
//!   and light `:active` while held. The runtime's press state is per-pointer, so
//!   resting one thumb on a key and pressing another with the other hand lights
//!   both and fires both.
//! - **A held key means something else.** `⌫` deletes one digit on a tap and
//!   clears everything when held — a touchscreen has one button and no
//!   modifiers, so holding is the only "other" gesture a key has. It fires while
//!   the finger is still down, and the lift is consumed rather than also
//!   deleting a digit.
//! - **A flickable tape.** The history is a kit-owned virtual list: drag it with
//!   a finger, flick it and it coasts to a stop, catch it and it stops dead
//!   without also recalling whatever line was sliding past. Tapping a line
//!   recalls its result; holding a line deletes it.
//! - **Insets.** The root pads itself by `env(safe-area-inset-*)`, so on a device
//!   with a notch or a gesture bar the keys are not under them. On a desktop
//!   every inset is zero and the padding costs nothing.
//!
//! Run it on a real machine (a display + a GPU adapter are required):
//! `cargo run -p obscur-kit --example calculator --features full`
//!
//! It cannot run in the headless dev/CI environment (no display, no GPU), so
//! there it is only compiled. The arithmetic is a plain state machine and *is*
//! testable there — `cargo test -p obscur-kit --features full --example
//! calculator` runs the cases at the bottom of this file. An example's tests do
//! not run in the workspace sweep, so that command is the only thing that runs
//! them.
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use obscur_kit::App;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::primitives::{Container, Text};
use obscur_kit::view::view::{EVENT_LONG_PRESS, View, ViewEventBinding, ViewNode};
use obscur_kit::widgets::button::Button;

/// The history tape's registered name.
const TAPE: InternedSymbol = InternedSymbol::named("tape");

/// The tape's row height, matching `.tape-row` in the sheet. The runtime sizes
/// its window and spacers from this.
const TAPE_ROW: f32 = 28.0;

/// Held `⌫` clears everything; tapped, it deletes one digit.
const CMD_CLEAR_ALL: InternedSymbol = InternedSymbol::named("clearAll");

/// The styling. `env(safe-area-inset-*)` on the root is the whole of this app's
/// awareness of the device it is on: a notched phone pads itself, a desktop
/// window does not.
const CSS: &str = "\
.app        { background-color: #11151f; width: 100%; height: 100%; \
              display: flex; flex-direction: column; gap: 8px; \
              padding-top: env(safe-area-inset-top); \
              padding-bottom: env(safe-area-inset-bottom); \
              padding-left: env(safe-area-inset-left); \
              padding-right: env(safe-area-inset-right); }\n\
.tape       { background-color: #161b28; width: 344px; height: 140px; \
              display: flex; flex-direction: column; overflow: scroll; }\n\
.tape-row   { width: 344px; height: 28px; display: flex; flex-direction: row; \
              overflow: hidden; }\n\
.tape-text  { color: #7f8db0; width: 336px; height: 22px; \
              white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }\n\
.tape-empty { color: #46506b; width: 336px; height: 22px; }\n\
.display    { background-color: #161b28; width: 344px; height: 72px; \
              display: flex; flex-direction: column; }\n\
.value      { color: #e9edf7; width: 336px; height: 40px; text-align: right; }\n\
.pending    { color: #6f7ea6; width: 336px; height: 20px; text-align: right; }\n\
.krow       { width: 344px; height: 68px; display: flex; flex-direction: row; \
              gap: 8px; }\n\
.key        { background-color: #232b3d; width: 80px; height: 68px; color: #e9edf7; \
              display: flex; justify-content: center; align-items: center; }\n\
.key:hover  { background-color: #2c3550; }\n\
.key:active { background-color: #4a5878; }\n\
.wide       { width: 168px; }\n\
.op         { background-color: #2a3352; }\n\
.op:active  { background-color: #55659a; }\n\
.eq         { background-color: #3b6ea5; }\n\
.eq:active  { background-color: #5b93cf; }\n\
.fn         { background-color: #1d2434; }\n\
.fn:active  { background-color: #3c4762; }\n\
";

/// What a key does. The label and the command name live beside it in [`KEYS`],
/// so adding a key is one row of that table and one arm of [`Calc::press`].
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
    /// The operator's symbol as the tape writes it.
    const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "−",
            Self::Mul => "×",
            Self::Div => "÷",
            _ => "",
        }
    }

    /// Applies this operator to `a` and `b`.
    ///
    /// Division by zero yields infinity rather than an error state: this is a
    /// demo of input handling, and an arithmetic error model would be the larger
    /// half of it.
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

/// Every key: `(command name, label, key, extra classes)`.
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

/// The keypad, four keys per row — except the last, where `0` is double width.
const ROWS: &[&[usize]] = &[
    &[0, 1, 2, 3],
    &[4, 5, 6, 7],
    &[8, 9, 10, 11],
    &[12, 13, 14, 15],
    &[16, 17, 18],
];

/// The most digits an entry accepts, so a thumb resting on `7` cannot grow the
/// string without bound.
const MAX_DIGITS: usize = 12;

/// The calculator's whole state.
struct Calc {
    /// The number being typed, as text — kept as text so `1.` and `1.0` survive
    /// while the user is still typing them.
    entry: String,
    /// The left-hand side of a pending operation.
    acc: Option<f64>,
    /// The pending operator.
    op: Option<Key>,
    /// Whether the next digit starts a fresh entry (the previous one was a
    /// result, or an operator was just pressed).
    fresh: bool,
    /// The history, newest first — so the interesting end needs no scrolling.
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

    /// The entry as a number.
    fn value(&self) -> f64 {
        self.entry.parse().unwrap_or(0.0)
    }

    /// What the tape and the display show for `value` — trimmed of the trailing
    /// zeroes a `f64` format leaves behind.
    fn format(value: f64) -> String {
        if value.is_finite() && value.fract() == 0.0 && value.abs() < 1e15 {
            format!("{value:.0}")
        } else {
            let text = format!("{value:.6}");
            let trimmed = text.trim_end_matches('0').trim_end_matches('.');
            trimmed.to_owned()
        }
    }

    /// The line under the display: the pending left-hand side and operator, if
    /// any. Empty when nothing is pending.
    fn pending(&self) -> String {
        match (self.acc, self.op) {
            (Some(acc), Some(op)) => format!("{} {}", Self::format(acc), op.symbol()),
            _ => String::new(),
        }
    }

    /// Handles one key.
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

    /// Folds the pending operation into the accumulator, so chaining
    /// `2 + 3 + 4` shows `5` when the second `+` is pressed.
    fn fold(&mut self) {
        let value = self.value();
        self.acc = Some(match (self.acc, self.op) {
            (Some(acc), Some(op)) if !self.fresh => op.apply(acc, value),
            (Some(acc), _) if self.fresh => acc,
            _ => value,
        });
    }

    /// Held `⌫`: back to nothing at all.
    fn clear_all(&mut self) {
        "0".clone_into(&mut self.entry);
        self.acc = None;
        self.op = None;
        self.fresh = true;
    }

    /// A tapped tape line recalls its result as the current entry.
    fn recall(&mut self, index: usize) {
        let Some(line) = self.tape.get(index) else {
            return;
        };
        if let Some((_, result)) = line.rsplit_once(" = ") {
            result.clone_into(&mut self.entry);
            self.fresh = false;
        }
    }

    /// A held tape line is deleted.
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

/// The whole screen: tape, display, keypad.
fn build_view(state: &Calc) -> View {
    let root = Container::new()
        .child(tape(state))
        .child(display(state))
        .with_children(ROWS.iter().map(|row| key_row(row)).collect::<Vec<_>>())
        .into_view_node()
        .class("app");
    View::single(root)
}

/// The scrolling history. Empty, it says so; otherwise it carries the virtual
/// list marker and the runtime realizes only the rows that fit.
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

/// One realized tape row. Built per visible index by the runtime, never for the
/// whole history.
fn tape_row(state: &Calc, index: usize) -> ViewNode {
    let text = state.tape.get(index).cloned().unwrap_or_default();
    Container::new()
        .child(Text::new(text).into_view_node().class("tape-text"))
        .into_view_node()
        .class("tape-row")
}

/// The current entry, with the pending left-hand side above it.
fn display(state: &Calc) -> ViewNode {
    Container::new()
        .child(Text::new(state.pending()).into_view_node().class("pending"))
        .child(
            Text::new(state.entry.clone())
                .into_view_node()
                .class("value"),
        )
        .into_view_node()
        .class("display")
}

/// One row of the keypad.
fn key_row(row: &[usize]) -> ViewNode {
    Container::new()
        .with_children(row.iter().map(|&i| key(i)).collect::<Vec<_>>())
        .into_view_node()
        .class("krow")
}

/// One key: a real [`Button`] bound to its command, with the sizing class on the
/// button's own node so the tap resolves to it.
///
/// `⌫` additionally binds `EVENT_LONG_PRESS`, which is what makes a held key a
/// different action rather than a slow tap.
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
