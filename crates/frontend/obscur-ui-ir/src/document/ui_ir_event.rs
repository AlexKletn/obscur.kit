#![doc = include_str!("ui_ir_event.doc/_module.md")]

#[doc = include_str!("ui_ir_event.doc/UiIrEvent.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UiIrEvent {
    event: Box<str>,
    handler: Box<str>,
}

impl UiIrEvent {
    #[doc = include_str!("ui_ir_event.doc/new.md")]
    #[must_use]
    pub fn new(event: impl Into<Box<str>>, handler: impl Into<Box<str>>) -> Self {
        Self {
            event: event.into(),
            handler: handler.into(),
        }
    }

    #[doc = include_str!("ui_ir_event.doc/event.md")]
    #[must_use]
    pub fn event(&self) -> &str {
        &self.event
    }

    #[doc = include_str!("ui_ir_event.doc/handler.md")]
    #[must_use]
    pub fn handler(&self) -> &str {
        &self.handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_return_wired_names() {
        let event = UiIrEvent::new("click", "submit");
        assert_eq!(event.event(), "click");
        assert_eq!(event.handler(), "submit");
    }

    #[test]
    fn equal_bindings_compare_equal() {
        assert_eq!(
            UiIrEvent::new("click", "submit"),
            UiIrEvent::new("click", "submit")
        );
        assert_ne!(
            UiIrEvent::new("click", "submit"),
            UiIrEvent::new("click", "cancel")
        );
    }
}
