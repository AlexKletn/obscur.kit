//! Maps a domain [`SemanticRole`] onto the closest AccessKit [`Role`] (§7.1).

use accesskit::Role;
use obscur_accessibility::semantic::SemanticRole;

/// Converts a domain [`SemanticRole`] into the closest AccessKit [`Role`]
/// (spec §7.1, §2.7).
///
/// The converter is stateless: it holds no configuration and every mapping is a
/// pure function of its input, so a single shared instance suffices. The domain
/// role set is deliberately smaller than AccessKit's, so each mapping picks the
/// nearest AccessKit control type; a handful are approximations, documented on
/// [`convert`](Self::convert).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RoleConverter;

impl RoleConverter {
    /// Creates the stateless converter.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Maps `role` to the closest AccessKit [`Role`].
    ///
    /// Most roles map one-to-one. The approximations are:
    /// - [`SemanticRole::Text`] → [`Role::Label`] (AccessKit's static-text
    ///   role);
    /// - [`SemanticRole::ProgressBar`] → [`Role::ProgressIndicator`];
    /// - [`SemanticRole::Separator`] → [`Role::Splitter`] (AccessKit has no
    ///   distinct separator role);
    /// - [`SemanticRole::Unknown`] → [`Role::Unknown`].
    #[must_use]
    pub const fn convert(&self, role: SemanticRole) -> Role {
        match role {
            SemanticRole::Unknown => Role::Unknown,
            SemanticRole::Group => Role::Group,
            SemanticRole::Region => Role::Region,
            SemanticRole::Button => Role::Button,
            SemanticRole::Link => Role::Link,
            SemanticRole::Text => Role::Label,
            SemanticRole::Heading => Role::Heading,
            SemanticRole::Image => Role::Image,
            SemanticRole::Checkbox => Role::CheckBox,
            SemanticRole::RadioButton => Role::RadioButton,
            SemanticRole::TextInput => Role::TextInput,
            SemanticRole::List => Role::List,
            SemanticRole::ListItem => Role::ListItem,
            SemanticRole::Table => Role::Table,
            SemanticRole::Row => Role::Row,
            SemanticRole::Cell => Role::Cell,
            SemanticRole::Dialog => Role::Dialog,
            SemanticRole::Menu => Role::Menu,
            SemanticRole::MenuItem => Role::MenuItem,
            SemanticRole::Tab => Role::Tab,
            SemanticRole::TabPanel => Role::TabPanel,
            SemanticRole::Slider => Role::Slider,
            SemanticRole::ProgressBar => Role::ProgressIndicator,
            SemanticRole::Separator => Role::Splitter,
            SemanticRole::Toolbar => Role::Toolbar,
            SemanticRole::Tooltip => Role::Tooltip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_to_one_mappings() {
        let c = RoleConverter::new();
        assert_eq!(c.convert(SemanticRole::Button), Role::Button);
        assert_eq!(c.convert(SemanticRole::Link), Role::Link);
        assert_eq!(c.convert(SemanticRole::Checkbox), Role::CheckBox);
        assert_eq!(c.convert(SemanticRole::RadioButton), Role::RadioButton);
        assert_eq!(c.convert(SemanticRole::TextInput), Role::TextInput);
        assert_eq!(c.convert(SemanticRole::Menu), Role::Menu);
        assert_eq!(c.convert(SemanticRole::MenuItem), Role::MenuItem);
        assert_eq!(c.convert(SemanticRole::Tab), Role::Tab);
        assert_eq!(c.convert(SemanticRole::TabPanel), Role::TabPanel);
    }

    #[test]
    fn documented_approximations() {
        let c = RoleConverter::new();
        assert_eq!(c.convert(SemanticRole::Text), Role::Label);
        assert_eq!(c.convert(SemanticRole::ProgressBar), Role::ProgressIndicator);
        assert_eq!(c.convert(SemanticRole::Separator), Role::Splitter);
        assert_eq!(c.convert(SemanticRole::Unknown), Role::Unknown);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(RoleConverter, RoleConverter::new());
    }
}
