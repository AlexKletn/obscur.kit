#![doc = include_str!("custom_values.doc/_module.md")]

use crate::orchestration::CustomValues;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("custom_values.doc/set_custom_value_parser.md")]
    pub fn set_custom_value_parser(&mut self, parser: CustomValues) {
        self.style.set_custom_values(parser);
    }

    #[doc = include_str!("custom_values.doc/has_custom_value_parser.md")]
    #[must_use]
    pub fn has_custom_value_parser(&self) -> bool {
        self.style.has_custom_values()
    }
}
