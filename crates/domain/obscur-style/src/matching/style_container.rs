use std::rc::Rc;

use obscur_identity::{FastMap, InternedSymbol};

use crate::property::{LengthContext, PropertyId, PropertyValue};

use super::custom_range_value::CustomRangeValue;

#[derive(Debug, PartialEq)]
pub(super) struct StyleContainer {
    pub(super) name: Option<Box<str>>,
    pub(super) properties: Rc<FastMap<InternedSymbol, PropertyValue>>,
    pub(super) values: FastMap<PropertyId, (PropertyId, PropertyValue)>,
    pub(super) ranges: FastMap<InternedSymbol, CustomRangeValue>,
    pub(super) range_context: Option<(LengthContext, bool)>,
    pub(super) font_size_context: Option<LengthContext>,
    pub(super) references: FastMap<(PropertyId, InternedSymbol), PropertyValue>,
}
