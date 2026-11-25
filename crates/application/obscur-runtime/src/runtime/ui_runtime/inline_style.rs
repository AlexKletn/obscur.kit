//! The `style` attribute's door into the runtime (spec §4.14, origin 5).

use crate::orchestration::InlineStyleCompiler;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("../../../docs/api/runtime/ui_runtime/inline_style.doc/set_inline_style_compiler.md")]
    pub fn set_inline_style_compiler(&mut self, compiler: InlineStyleCompiler) {
        self.inline_styles.install(compiler);
        self.last_style = None;
    }

    /// Whether a `style` attribute would be understood at all.
    #[must_use]
    pub fn has_inline_style_compiler(&self) -> bool {
        self.inline_styles.is_installed()
    }
}
