//! **Agent-style headless inspection of the showcase scene (spec §7.6).**
//!
//! Renders the shared showcase scene at a configurable viewport (env
//! `INSPECT_W` / `INSPECT_H`, default a tall tiled-window shape) and prints the
//! top of the tree via [`UiRuntime::inspect_snapshot`] — the same structured
//! observation an agent reads in a run, no window required.
//!
//! Run it: `cargo run -p obscur-kit --example showcase_inspect --features render-cpu`
#![forbid(unsafe_code)]

use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;

#[path = "showcase/scene.rs"]
mod showcase;

fn main() {
    let width: f32 = std::env::var("INSPECT_W").ok().and_then(|v| v.parse().ok()).unwrap_or(1478.0);
    let height: f32 = std::env::var("INSPECT_H").ok().and_then(|v| v.parse().ok()).unwrap_or(950.0);

    let src = CssSource::anonymous(showcase::stylesheet());
    let parsed = StylesheetParser::new(&src).parse();
    let sheet = StylesheetCompiler::new().compile(&parsed).matchable;

    let state = showcase::ShowcaseState::default();
    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(width, height));
    runtime.render_now(&showcase::view(&state), std::slice::from_ref(&sheet));

    println!("viewport {width}×{height} — top of the tree:");
    for node in runtime.inspect_snapshot().nodes {
        if node.depth <= 1 {
            println!("{node}");
        }
    }
}
