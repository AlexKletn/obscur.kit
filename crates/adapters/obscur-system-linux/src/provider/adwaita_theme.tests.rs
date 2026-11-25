use super::*;

#[test]
fn user_css_can_override_the_ordinary_window_radius() {
    let mut style = WindowRadius::default();
    style.cascade(
        ":root { --window-radius: 15px; } window.csd { border-radius: var(--window-radius); }
         window.csd.maximized, window.csd.tiled { --window-radius: 0px; }",
    );
    assert_eq!(style.resolve(), Some(15.0));
    style.cascade("/* window.csd { border-radius: 99px; } */ :root { --window-radius: 9.5px; }");
    assert_eq!(style.resolve(), Some(9.5));
    style.cascade("window.csd { border-radius: 0; }");
    assert_eq!(style.resolve(), Some(0.0));
}

#[test]
fn installed_adwaita_exposes_its_radius_when_the_resource_is_available() {
    if let Some(css) = super::super::adwaita_stylesheet::load() {
        let mut theme = WindowRadius::default();
        theme.cascade(css);
        assert!(theme.resolve().is_some());
    }
}
