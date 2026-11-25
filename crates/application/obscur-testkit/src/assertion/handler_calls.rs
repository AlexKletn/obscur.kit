//! A command handler that counts the times it ran.

use std::cell::Cell;
use std::rc::Rc;

#[doc = include_str!("../../docs/api/assertion/handler_calls.doc/counting_handler.md")]
pub fn counting_handler() -> (Rc<Cell<u32>>, impl FnMut() + 'static) {
    let count = Rc::new(Cell::new(0_u32));
    let counter = Rc::clone(&count);
    (count, move || counter.set(counter.get() + 1))
}
