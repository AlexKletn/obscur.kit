use std::sync::Arc;

use obscur_identity::InternedSymbol;

/// One token fragment in a deferred container shorthand.
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerComponent {
    /// Literal CSS tokens.
    Literal(Box<str>),
    /// A reference to a custom property with an optional token fallback.
    Variable {
        /// The custom property supplying the shorthand.
        name: InternedSymbol,
        /// The shorthand token fallback when the reference cannot be resolved.
        fallback: Option<Arc<str>>,
    },
    /// A variable whose fallback contains another variable reference.
    VariableWithTemplateFallback {
        /// The custom property supplying the shorthand.
        name: InternedSymbol,
        /// The fallback token fragments, resolved only when the variable is missing.
        fallback: Arc<[ContainerComponent]>,
    },
    /// A CSS function containing deferred variable substitutions.
    Function {
        /// The function identifier.
        name: Box<str>,
        /// The function argument tokens.
        args: Arc<[ContainerComponent]>,
    },
}
