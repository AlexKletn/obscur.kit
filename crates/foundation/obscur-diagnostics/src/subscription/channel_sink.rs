use crate::diagnostic::Diagnostic;

use super::DiagnosticChannel;

#[doc = include_str!("../../docs/api/subscription/diagnostic_channel.doc/ChannelSink.md")]
#[derive(Debug, Default)]
pub struct ChannelSink {
    channel: std::cell::RefCell<DiagnosticChannel>,
}

impl ChannelSink {
    /// Wraps `channel` as a sink.
    #[must_use]
    pub fn new(channel: DiagnosticChannel) -> Self {
        Self {
            channel: std::cell::RefCell::new(channel),
        }
    }

    /// Borrows the underlying channel mutably (e.g. to subscribe or drain).
    #[must_use]
    pub fn channel_mut(&self) -> std::cell::RefMut<'_, DiagnosticChannel> {
        self.channel.borrow_mut()
    }

    /// Consumes the sink, returning the underlying channel.
    #[must_use]
    pub fn into_inner(self) -> DiagnosticChannel {
        self.channel.into_inner()
    }
}

impl crate::port::DiagnosticSink for ChannelSink {
    fn emit(&self, diagnostic: &Diagnostic) {
        self.channel.borrow_mut().publish(diagnostic);
    }
}
