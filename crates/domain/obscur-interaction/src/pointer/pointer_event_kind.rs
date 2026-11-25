/// The lifecycle kind of a [`PointerEvent`](super::PointerEvent) (spec §4.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerEventKind {
    /// A button was pressed or a contact began.
    Down,
    /// A button was released or a contact ended.
    Up,
    /// The pointer moved within the target.
    Move,
    /// The pointer entered the target.
    Enter,
    /// The pointer left the target.
    Leave,
    /// The interaction was cancelled by the system (for example, capture lost).
    Cancel,
}

impl PointerEventKind {
    /// Returns whether this kind terminates the pointer's interaction with the
    /// target ([`Up`](Self::Up), [`Cancel`](Self::Cancel), or
    /// [`Leave`](Self::Leave)).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Up | Self::Cancel | Self::Leave)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_is_terminal() {
        assert!(PointerEventKind::Up.is_terminal());
        assert!(PointerEventKind::Cancel.is_terminal());
        assert!(PointerEventKind::Leave.is_terminal());
        assert!(!PointerEventKind::Down.is_terminal());
        assert!(!PointerEventKind::Move.is_terminal());
        assert!(!PointerEventKind::Enter.is_terminal());
    }
}
