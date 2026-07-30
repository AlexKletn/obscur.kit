//! A view written with the primary authoring surface, in a crate that has only
//! the facade.

use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;

/// The command the button below fires.
pub const CMD_INCREMENT: InternedSymbol = InternedSymbol::named("increment");

/// The §7.5 identifier the counter's readout carries, and the name an application
/// hands to `SelectorTarget::test_id` to reach that node before a `NodeId` exists.
pub const COUNT_TEST_ID: &str = "count-row";

/// The §7.5 identifier on the button, under the *other* reserved name — one
/// spelling per owner, as `stable_selector.rs` puts it.
pub const PRESS_AUTOMATION_ID: &str = "press-button";

/// A view built by the `okui!` macro over a value from the caller's own scope.
///
/// This function is the whole of audit finding 1-A: it did not compile, because
/// the expansion named `::obscur_view` and `::obscur_identity` — crates this one
/// does not (and must not) depend on.
///
/// It carries audit finding 2-3 too. `test-id="count-row"` is three tokens to
/// Rust (`test`, `-`, `id`), so this markup was `expected ident` and §7.5's
/// identifiers — the only way an application names a node to
/// `App::on_command_at` before the first frame — were authorable in a `.okui`
/// file and nowhere else. The hyphenated names below are the assertion; the test
/// only has to look them up.
#[must_use]
pub fn counter_view(count: i32) -> View {
    let label = format!("count: {count}");
    okui! {
        <div class="app">
            <div class="count" test-id="count-row"><span>{label}</span></div>
            <button class="btn" automation-id="press-button" on:click={increment}>
                <span>"press"</span>
            </button>
        </div>
    }
}
