#![doc = include_str!("diagnostic_subsystem.doc/_module.md")]

#[doc = include_str!("diagnostic_subsystem.doc/DiagnosticSubsystem.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSubsystem {
    #[doc = include_str!("diagnostic_subsystem.doc/View.md")]
    View,
    #[doc = include_str!("diagnostic_subsystem.doc/Tree.md")]
    Tree,
    #[doc = include_str!("diagnostic_subsystem.doc/Style.md")]
    Style,
    #[doc = include_str!("diagnostic_subsystem.doc/Layout.md")]
    Layout,
    #[doc = include_str!("diagnostic_subsystem.doc/Text.md")]
    Text,
    #[doc = include_str!("diagnostic_subsystem.doc/Motion.md")]
    Motion,
    #[doc = include_str!("diagnostic_subsystem.doc/Render.md")]
    Render,
    #[doc = include_str!("diagnostic_subsystem.doc/Resources.md")]
    Resources,
    #[doc = include_str!("diagnostic_subsystem.doc/Platform.md")]
    Platform,
    #[doc = include_str!("diagnostic_subsystem.doc/Plugin.md")]
    Plugin,
    #[doc = include_str!("diagnostic_subsystem.doc/Runtime.md")]
    Runtime,
}
