//! Queries the accessibility projection of a rendered runtime.

use obscur_accessibility::semantic::SemanticRole;
use obscur_geometry::Size;
use obscur_identity::RuntimeId;
use obscur_runtime::UiRuntime;
use obscur_testkit::{fixture::ButtonElement, selector::SemanticSelector};

#[test]
fn queries_the_rendered_button_by_role_and_accessible_name() {
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(120.0, 80.0));
    runtime.render_now(&ButtonElement::view(), &[]);
    let matches = SemanticSelector::by_role(&runtime, SemanticRole::Button, Some("Save"));
    assert_eq!(matches.len(), 1);
    assert!(runtime.tree().get(matches[0]).is_some());
    assert_eq!(
        SemanticSelector::by_role(&runtime, SemanticRole::Button, None),
        matches
    );
    assert!(SemanticSelector::by_role(&runtime, SemanticRole::Button, Some("Cancel")).is_empty());
}
