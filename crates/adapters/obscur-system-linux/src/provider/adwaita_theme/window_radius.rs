use super::border_radius::{BorderRadius, pixels};

#[derive(Default)]
pub(super) struct WindowRadius {
    root: Option<f32>,
    window: Option<f32>,
    csd: Option<f32>,
    border: Option<BorderRadius>,
}

impl WindowRadius {
    pub(super) fn cascade(&mut self, stylesheet: &str) {
        let stylesheet = without_comments(stylesheet);
        for rule in stylesheet.split('}') {
            let Some((selectors, declarations)) = rule.split_once('{') else {
                continue;
            };
            let selectors = selectors.rsplit(';').next().unwrap_or(selectors);
            for selector in selectors.split(',').map(str::trim) {
                let variable = match selector {
                    ":root" => &mut self.root,
                    "window" => &mut self.window,
                    "window.csd" => &mut self.csd,
                    _ => continue,
                };
                for (property, value) in declarations
                    .split(';')
                    .filter_map(|part| part.split_once(':'))
                {
                    match property.trim() {
                        "--window-radius" => {
                            if let Some(radius) = pixels(value) {
                                *variable = Some(radius);
                            }
                        }
                        "border-radius" if selector == "window.csd" => {
                            if let Some(radius) = BorderRadius::parse(value) {
                                self.border = Some(radius);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub(super) fn resolve(&self) -> Option<f32> {
        match self.border {
            Some(BorderRadius::Pixels(value)) => Some(value),
            Some(BorderRadius::WindowVariable) => self.csd.or(self.window).or(self.root),
            None => None,
        }
    }
}

fn without_comments(mut css: &str) -> String {
    let mut result = String::new();
    while let Some((before, comment)) = css.split_once("/*") {
        result.push_str(before);
        result.push(' ');
        let Some((_, after)) = comment.split_once("*/") else {
            return result;
        };
        css = after;
    }
    result.push_str(css);
    result
}
