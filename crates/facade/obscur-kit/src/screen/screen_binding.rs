use std::sync::Arc;

use crate::view::view::ViewValue;

use super::ScreenRow;

#[derive(Clone)]
pub(super) enum ScreenBinding {
    Value(ViewValue),
    Rows(Arc<[ScreenRow]>),
}

impl ScreenBinding {
    pub(super) fn value(&self) -> Option<ViewValue> {
        match self {
            Self::Value(value) => Some(value.clone()),
            Self::Rows(_) => None,
        }
    }
}
