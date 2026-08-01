#![doc = include_str!("semantic_role.doc/_module.md")]

#[doc = include_str!("semantic_role.doc/SemanticRole.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SemanticRole {
    #[doc = include_str!("semantic_role.doc/Unknown.md")]
    #[default]
    Unknown,
    #[doc = include_str!("semantic_role.doc/Group.md")]
    Group,
    #[doc = include_str!("semantic_role.doc/Region.md")]
    Region,
    #[doc = include_str!("semantic_role.doc/Button.md")]
    Button,
    #[doc = include_str!("semantic_role.doc/Link.md")]
    Link,
    #[doc = include_str!("semantic_role.doc/Text.md")]
    Text,
    #[doc = include_str!("semantic_role.doc/Heading.md")]
    Heading,
    #[doc = include_str!("semantic_role.doc/Image.md")]
    Image,
    #[doc = include_str!("semantic_role.doc/Checkbox.md")]
    Checkbox,
    #[doc = include_str!("semantic_role.doc/RadioButton.md")]
    RadioButton,
    #[doc = include_str!("semantic_role.doc/TextInput.md")]
    TextInput,
    #[doc = include_str!("semantic_role.doc/List.md")]
    List,
    #[doc = include_str!("semantic_role.doc/ListItem.md")]
    ListItem,
    #[doc = include_str!("semantic_role.doc/Table.md")]
    Table,
    #[doc = include_str!("semantic_role.doc/Row.md")]
    Row,
    #[doc = include_str!("semantic_role.doc/Cell.md")]
    Cell,
    #[doc = include_str!("semantic_role.doc/Dialog.md")]
    Dialog,
    #[doc = include_str!("semantic_role.doc/Menu.md")]
    Menu,
    #[doc = include_str!("semantic_role.doc/MenuItem.md")]
    MenuItem,
    #[doc = include_str!("semantic_role.doc/Tab.md")]
    Tab,
    #[doc = include_str!("semantic_role.doc/TabPanel.md")]
    TabPanel,
    #[doc = include_str!("semantic_role.doc/Slider.md")]
    Slider,
    #[doc = include_str!("semantic_role.doc/ProgressBar.md")]
    ProgressBar,
    #[doc = include_str!("semantic_role.doc/Separator.md")]
    Separator,
    #[doc = include_str!("semantic_role.doc/Toolbar.md")]
    Toolbar,
    #[doc = include_str!("semantic_role.doc/Tooltip.md")]
    Tooltip,
}

impl SemanticRole {
    #[doc = include_str!("semantic_role.doc/is_interactive.md")]
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

    #[doc = include_str!("semantic_role.doc/is_container.md")]
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
