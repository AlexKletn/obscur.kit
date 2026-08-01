#![doc = include_str!("counter.doc/_module.md")]

use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;

#[doc = include_str!("counter.doc/CMD_INCREMENT.md")]
pub const CMD_INCREMENT: InternedSymbol = InternedSymbol::named("increment");

#[doc = include_str!("counter.doc/COUNT_TEST_ID.md")]
pub const COUNT_TEST_ID: &str = "count-row";

#[doc = include_str!("counter.doc/PRESS_AUTOMATION_ID.md")]
pub const PRESS_AUTOMATION_ID: &str = "press-button";

#[doc = include_str!("counter.doc/counter_view.md")]
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
