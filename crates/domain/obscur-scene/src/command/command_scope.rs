use super::PaintCommand;

/// The kind of state opened by a paint command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CommandScope {
    Transform,
    Clip,
    Layer,
}

impl CommandScope {
    pub(super) const fn from_push(command: &PaintCommand) -> Option<Self> {
        match command {
            PaintCommand::PushTransform(_) => Some(Self::Transform),
            PaintCommand::PushClip(_) => Some(Self::Clip),
            PaintCommand::PushLayer(_) => Some(Self::Layer),
            _ => None,
        }
    }

    pub(super) const fn close(self) -> PaintCommand {
        match self {
            Self::Transform => PaintCommand::PopTransform,
            Self::Clip => PaintCommand::PopClip,
            Self::Layer => PaintCommand::PopLayer,
        }
    }
}
