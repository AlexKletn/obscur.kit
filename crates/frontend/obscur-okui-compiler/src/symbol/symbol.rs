#![doc = include_str!("symbol.doc/_module.md")]

#[doc = include_str!("symbol.doc/SymbolKind.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    #[doc = include_str!("symbol.doc/Prop.md")]
    Prop,
    #[doc = include_str!("symbol.doc/State.md")]
    State,
    #[doc = include_str!("symbol.doc/LoopLocal.md")]
    LoopLocal,
    #[doc = include_str!("symbol.doc/Component.md")]
    Component,
    #[doc = include_str!("symbol.doc/Function.md")]
    Function,
    #[doc = include_str!("symbol.doc/Slot.md")]
    Slot,
}

impl SymbolKind {
    #[doc = include_str!("symbol.doc/is_reference_target.md")]
    #[must_use]
    pub const fn is_reference_target(&self) -> bool {
        matches!(self, Self::Prop | Self::State | Self::LoopLocal)
    }
}

#[doc = include_str!("symbol.doc/Symbol.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    name: Box<str>,
    kind: SymbolKind,
}

impl Symbol {
    #[doc = include_str!("symbol.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>, kind: SymbolKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }

    #[doc = include_str!("symbol.doc/prop.md")]
    #[must_use]
    pub fn prop(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::Prop)
    }

    #[doc = include_str!("symbol.doc/state.md")]
    #[must_use]
    pub fn state(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::State)
    }

    #[doc = include_str!("symbol.doc/loop_local.md")]
    #[must_use]
    pub fn loop_local(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::LoopLocal)
    }

    #[doc = include_str!("symbol.doc/component.md")]
    #[must_use]
    pub fn component(name: impl Into<Box<str>>) -> Self {
        Self::new(name, SymbolKind::Component)
    }

    #[doc = include_str!("symbol.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("symbol.doc/kind.md")]
    #[must_use]
    pub const fn kind(&self) -> &SymbolKind {
        &self.kind
    }

    #[doc = include_str!("symbol.doc/is_reference_target_2.md")]
    #[must_use]
    pub const fn is_reference_target(&self) -> bool {
        self.kind.is_reference_target()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_set_name_and_kind() {
        assert_eq!(Symbol::prop("count").name(), "count");
        assert_eq!(Symbol::prop("count").kind(), &SymbolKind::Prop);
        assert_eq!(Symbol::state("open").kind(), &SymbolKind::State);
        assert_eq!(Symbol::loop_local("item").kind(), &SymbolKind::LoopLocal);
        assert_eq!(Symbol::component("Row").kind(), &SymbolKind::Component);
        assert_eq!(
            Symbol::new("f", SymbolKind::Function).kind(),
            &SymbolKind::Function
        );
    }

    #[test]
    fn is_reference_target_only_for_value_bindings() {
        assert!(Symbol::prop("a").is_reference_target());
        assert!(Symbol::state("b").is_reference_target());
        assert!(Symbol::loop_local("item").is_reference_target());
        assert!(!Symbol::component("C").is_reference_target());
        assert!(!Symbol::new("f", SymbolKind::Function).is_reference_target());
        assert!(!Symbol::new("s", SymbolKind::Slot).is_reference_target());
    }
}
