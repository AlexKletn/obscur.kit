//! The ARIA-like semantic role of an accessibility node (spec §7.1).

/// The semantic role of an accessibility node (spec §7.1).
///
/// A role is the ARIA-like classification a platform adapter maps onto its
/// native accessibility API (UI Automation control types, AX roles, AT-SPI
/// roles, or web `role` attributes). It covers the common cross-platform set;
/// [`Unknown`](Self::Unknown) is the neutral default for nodes whose role has
/// not been established.
///
/// Two coarse predicates classify roles without exhaustive matching:
/// [`is_interactive`](Self::is_interactive) for roles a user can operate, and
/// [`is_container`](Self::is_container) for roles that primarily group other
/// nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SemanticRole {
    /// An unclassified node. The default.
    #[default]
    Unknown,
    /// A generic grouping of related nodes with no stronger semantics.
    Group,
    /// A landmark region of the interface.
    Region,
    /// A push button that performs an action when activated.
    Button,
    /// A hyperlink to another location or resource.
    Link,
    /// A run of static, non-editable text.
    Text,
    /// A heading that introduces a section of content.
    Heading,
    /// A graphical image.
    Image,
    /// A two- or tri-state checkbox.
    Checkbox,
    /// A radio button, one choice within a mutually exclusive group.
    RadioButton,
    /// An editable text field.
    TextInput,
    /// A list of items.
    List,
    /// A single item within a list.
    ListItem,
    /// A data table.
    Table,
    /// A row within a table.
    Row,
    /// A cell within a table row.
    Cell,
    /// A dialog or modal surface.
    Dialog,
    /// A menu of choices.
    Menu,
    /// A single item within a menu.
    MenuItem,
    /// A tab in a tab strip.
    Tab,
    /// The content panel associated with a tab.
    TabPanel,
    /// A slider for selecting a value from a range.
    Slider,
    /// A progress indicator.
    ProgressBar,
    /// A visual or semantic separator between content.
    Separator,
    /// A toolbar grouping controls.
    Toolbar,
    /// A tooltip providing contextual help.
    Tooltip,
}

impl SemanticRole {
    /// Returns whether this role names a control the user can directly operate
    /// (spec §7.1): buttons, links, checkboxes, radio buttons, text inputs,
    /// menu items, tabs, and sliders.
    #[must_use]
    pub const fn is_interactive(&self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::Link
                | Self::Checkbox
                | Self::RadioButton
                | Self::TextInput
                | Self::MenuItem
                | Self::Tab
                | Self::Slider
        )
    }

    /// Returns whether this role primarily groups other nodes (spec §7.1):
    /// groups, regions, lists, tables, rows, dialogs, menus, toolbars, and tab
    /// panels.
    #[must_use]
    pub const fn is_container(&self) -> bool {
        matches!(
            self,
            Self::Group
                | Self::Region
                | Self::List
                | Self::Table
                | Self::Row
                | Self::Dialog
                | Self::Menu
                | Self::Toolbar
                | Self::TabPanel
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unknown() {
        assert_eq!(SemanticRole::default(), SemanticRole::Unknown);
    }

    #[test]
    fn interactive_roles() {
        for role in [
            SemanticRole::Button,
            SemanticRole::Link,
            SemanticRole::Checkbox,
            SemanticRole::RadioButton,
            SemanticRole::TextInput,
            SemanticRole::MenuItem,
            SemanticRole::Tab,
            SemanticRole::Slider,
        ] {
            assert!(role.is_interactive(), "{role:?} should be interactive");
        }
        assert!(!SemanticRole::Text.is_interactive());
        assert!(!SemanticRole::Group.is_interactive());
    }

    #[test]
    fn container_roles() {
        for role in [
            SemanticRole::Group,
            SemanticRole::Region,
            SemanticRole::List,
            SemanticRole::Table,
            SemanticRole::Row,
            SemanticRole::Dialog,
            SemanticRole::Menu,
            SemanticRole::Toolbar,
            SemanticRole::TabPanel,
        ] {
            assert!(role.is_container(), "{role:?} should be a container");
        }
        assert!(!SemanticRole::Button.is_container());
        assert!(!SemanticRole::Text.is_container());
    }
}
