#![doc = include_str!("inline_style.doc/_module.md")]

use crate::orchestration::InlineStyleCompiler;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("inline_style.doc/set_inline_style_compiler.md")]
    pub fn set_inline_style_compiler(&mut self, compiler: InlineStyleCompiler) {
        self.inline_styles.install(compiler);
    }

    #[doc = include_str!("inline_style.doc/has_inline_style_compiler.md")]
    #[must_use]
    pub fn has_inline_style_compiler(&self) -> bool {
        self.inline_styles.is_installed()
    }
}
