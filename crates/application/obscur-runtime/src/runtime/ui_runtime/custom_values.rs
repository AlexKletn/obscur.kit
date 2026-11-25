#![doc = include_str!("../../../docs/api/runtime/ui_runtime/custom_values.doc/_module.md")]

use crate::orchestration::CustomValues;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/custom_values.doc/set_custom_value_parser.md")]
    pub fn set_custom_value_parser(&mut self, parser: CustomValues) {
        self.style.set_custom_values(parser);
    }

    /// Whether a custom property's declared text can be read at all.
    #[must_use]
    pub fn has_custom_value_parser(&self) -> bool {
        self.style.has_custom_values()
    }
}
