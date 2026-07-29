//! The boundary claims: what an application must be able to do with the facade
//! as its only dependency.
//!
//! Compiling this file at all is most of the assertion — see `src/counter.rs`.
#![forbid(unsafe_code)]

use obscur_kit::identity::InternedSymbol;

use facade_consumer::counter::{CMD_INCREMENT, counter_view};
use facade_consumer::measure::{find, measure};

const CSS: &str = ".app { width: 100%; height: 100% } .count { height: 20px }";

#[test]
fn the_okui_macro_expands_in_a_crate_that_has_only_the_facade() {
    // Reaching the view proves the expansion produced a real one rather than an
    // empty fragment.
    let view = counter_view(7);
    assert_eq!(view.root().len(), 1);
}

#[test]
fn a_view_the_macro_built_lays_out_through_the_facade() {
    let nodes = measure(&counter_view(1), CSS, 480.0, 240.0);
    let app = find(&nodes, "app").expect("the authored root is in the snapshot");
    assert_eq!((app.width, app.height), (480.0, 240.0));
}

#[test]
fn an_application_can_name_a_command_the_markup_binds_to() {
    // The name in `on:click={increment}` and the symbol an application registers a
    // handler under have to be the same handle, and an application can only reach
    // the latter through the facade.
    assert_eq!(CMD_INCREMENT, InternedSymbol::named("increment"));
}
