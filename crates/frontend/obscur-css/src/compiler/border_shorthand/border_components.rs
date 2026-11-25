use obscur_style::property::PropertyValue;

/// The `width` / `style` / `color` sub-values parsed from a `border` /
/// `border-<side>` shorthand value, each present only when authored.
pub(super) struct BorderComponents {
    pub(super) width: Option<PropertyValue>,
    pub(super) style: Option<PropertyValue>,
    pub(super) color: Option<PropertyValue>,
}
