//! A view written with the primary authoring surface, in a crate that has only
//! the facade.

use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;

/// The command the button below fires.
pub const CMD_INCREMENT: InternedSymbol = InternedSymbol::named("increment");

/// A view built by the `okui!` macro over a value from the caller's own scope.
///
/// This function is the whole of audit finding 1-A: it did not compile, because
/// the expansion named `::obscur_view` and `::obscur_identity` — crates this one
/// does not (and must not) depend on.
#[must_use]
pub fn counter_view(count: i32) -> View {
    let label = format!("count: {count}");
    okui! {
        <div class="app">
            <div class="count"><span>{label}</span></div>
            <button class="btn" on:click={increment}><span>"press"</span></button>
        </div>
    }
}
