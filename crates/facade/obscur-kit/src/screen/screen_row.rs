use std::collections::HashMap;

use crate::view::view::ViewValue;

/// A record in a runtime-loaded screen collection. The row template reads these fields through its item binding.
pub type ScreenRow = HashMap<Box<str>, ViewValue>;
