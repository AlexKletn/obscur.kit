//! A minimal **file manager**, authored in `okui!` markup over real data — the
//! demo that exercises the declarative entry point end to end.
//!
//! What it proves, concretely:
//!
//! * [`App`] boots a window in a handful of lines (see `main`).
//! * `okui!` markup binds **real Rust values**: `{row.name}` renders the actual
//!   file name, `class={row.class}` drives the selection highlight, and
//!   `<For each={rows}>` emits one node per directory entry — a real Rust loop,
//!   not a template materialized once. (A standalone `.okui` *file* cannot do
//!   this today: the runtime bridge lowers a binding to its path's spelling and
//!   materializes `<For>` once. The inline macro resolves bindings in *this*
//!   function's scope, which is why the markup lives here.)
//! * The kit performs no I/O of its own (§5.15): `std::fs` reads the directory,
//!   the kit only renders. No network, no new dependency.
//!
//! Navigation is **selection-based** — the toolbar moves a cursor and opens the
//! selected entry — because an `on:click=name` handler compiles to one symbol per
//! markup site, so every row rendered from the same `<For>` body would share it.
//! Per-row click targets need either a payload on the command or a distinct symbol
//! per row; neither exists yet.
//!
//! Run it (a display and a GPU adapter are required):
//! `cargo run -p obscur-kit --example file_manager --features full`
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_ir::name_binding::fallback_symbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::View;
use obscur_kit::{App, FrameInfo};

/// Row height, and the chrome above and below the list — the CSS below and this
/// arithmetic have to agree, which is the honest cost of app-side virtualization.
const ROW_H: f32 = 24.0;
/// Toolbar height, per the sheet.
const TOOLBAR_H: f32 = 46.0;
/// Status bar height, per the sheet.
const STATUS_H: f32 = 24.0;

/// The styling. Authored CSS, installed through the motion-aware pipeline by
/// [`App::stylesheet`].
const CSS: &str = "\
.app      { background-color: #12161f; width: 100%; height: 100%; \
            display: flex; flex-direction: column; }\n\
.toolbar  { background-color: #1b2130; width: 100%; height: 46px; \
            display: flex; flex-direction: row; gap: 8px; }\n\
.btn      { background-color: #2b3348; width: 96px; height: 30px; }\n\
.btn:hover{ background-color: #3d4a6b; }\n\
.btn-label{ color: #e6e9f0; width: 100%; height: 20px; }\n\
.path     { color: #9fb0d0; width: 100%; height: 30px; }\n\
.list     { width: 100%; height: 100%; display: flex; flex-direction: column; }\n\
.row      { width: 100%; height: 24px; display: flex; flex-direction: row; gap: 10px; }\n\
.row:hover{ background-color: #1b2130; }\n\
.sel      { background-color: #2b3f66; }\n\
.kind     { color: #7f8db0; width: 52px; height: 20px; }\n\
.name     { color: #e6e9f0; width: 100%; height: 20px; }\n\
.size     { color: #9fb0d0; width: 96px; height: 20px; }\n\
.status   { background-color: #1b2130; width: 100%; height: 24px; }\n\
.hint     { color: #7f8db0; width: 100%; height: 20px; }\n\
";

/// The same sheet with every `background-color` dropped — an overdraw probe.
///
/// The normal sheet stacks opaque fills: `.app` covers the window, `.list` covers
/// it again, and each of the ~76 rows paints its own. A path renderer that does no
/// occlusion culling pays for all of them. Running with `OBSCUR_NO_BG=1` renders the
/// identical tree and the identical text with none of those fills, so the difference
/// in `present` is the cost of the overdraw rather than of the geometry.
const CSS_NO_BG: &str = "\
.app      { width: 100%; height: 100%; display: flex; flex-direction: column; }\n\
.toolbar  { width: 100%; height: 46px; display: flex; flex-direction: row; gap: 8px; }\n\
.btn      { width: 96px; height: 30px; }\n\
.btn-label{ color: #e6e9f0; width: 100%; height: 20px; }\n\
.path     { color: #9fb0d0; width: 100%; height: 30px; }\n\
.list     { width: 100%; height: 100%; display: flex; flex-direction: column; }\n\
.row      { width: 100%; height: 24px; display: flex; flex-direction: row; gap: 10px; }\n\
.kind     { color: #7f8db0; width: 52px; height: 20px; }\n\
.name     { color: #e6e9f0; width: 100%; height: 20px; }\n\
.size     { color: #9fb0d0; width: 96px; height: 20px; }\n\
.status   { width: 100%; height: 24px; }\n\
.hint     { color: #7f8db0; width: 100%; height: 20px; }\n\
";

/// Whether to blank every string the UI renders (`OBSCUR_NO_TEXT=1`).
///
/// A glyph probe. The tree, the node count, the classes and the layout stay
/// **identical** — only the text content becomes empty, so no glyph is shaped or
/// rasterized. The difference in `present` is therefore the cost of the text alone,
/// with nothing else moving. Vello draws glyphs as filled paths rather than from an
/// atlas, and this list carries ~230 runs a frame.
fn blank_text() -> bool {
    std::env::var_os("OBSCUR_NO_TEXT").is_some()
}

/// `text` unless the glyph probe is on, in which case the empty string.
fn maybe(text: String) -> String {
    if blank_text() { String::new() } else { text }
}

/// One directory entry as the UI needs it — every field already a `String`, so the
/// markup binds them directly.
struct Row {
    /// The reconciler's key. ALWAYS the real name, never blanked: the glyph probe
    /// empties what is *displayed*, and if it emptied this too every row would share
    /// the key `""`. 76 duplicate keys make keyed matching thrash — a full restyle
    /// and a full layout every frame — which looks exactly like a layout bug and is
    /// not one.
    key: String,
    name: String,
    kind: String,
    size: String,
    /// `"row"`, or `"row sel"` for the cursor line — bound to `class=`.
    class: String,
}

/// The browser's state: where we are, what is there, and which line is selected.
struct State {
    dir: PathBuf,
    entries: Vec<Entry>,
    selected: usize,
    message: String,
    /// The app's own scroll position, in rows. Virtualization means the kit never
    /// sees the rows outside the window, so it cannot scroll them — the app owns the
    /// position and decides which slice exists at all.
    first_visible: f32,
}

/// A directory entry as read from the filesystem.
struct Entry {
    name: String,
    is_dir: bool,
    size: u64,
}

impl State {
    /// Reads `dir`, sorting directories first and then by name. A read failure is
    /// reported in the status line and leaves the previous listing in place — the
    /// §1.x recoverable-failure posture, not a panic.
    fn load(&mut self, dir: PathBuf) {
        match read_entries(&dir) {
            Ok(entries) => {
                self.dir = dir;
                self.entries = entries;
                self.selected = 0;
                self.first_visible = 0.0;
                self.message = format!("{} entries", self.entries.len());
                if blank_text() {
                    // The probe blanks the UI, so state that would be on screen has
                    // to be said out loud: this is what layout is actually sizing.
                    println!(
                        "[glyph-probe] {} entries in {}",
                        self.entries.len(),
                        self.dir.display()
                    );
                }
            }
            Err(error) => self.message = format!("cannot open: {error}"),
        }
    }

    /// Only the rows the viewport can show, shaped for the markup.
    ///
    /// This is the whole point of virtualizing: style, layout and compose each cost
    /// per node, so a directory with 10,000 entries is only affordable if 10,000
    /// nodes are never built. The window is derived from the viewport the runtime
    /// reports, so a resize widens it and nothing else has to know.
    fn rows(&self, visible: usize) -> Vec<Row> {
        let first = self.first_visible as usize;
        self.entries
            .iter()
            .enumerate()
            .skip(first)
            .take(visible)
            .map(|(index, entry)| Row {
                key: entry.name.clone(),
                name: maybe(entry.name.clone()),
                kind: maybe(if entry.is_dir { "DIR" } else { "file" }.to_owned()),
                size: maybe(if entry.is_dir {
                    String::new()
                } else {
                    human_size(entry.size)
                }),
                class: if index == self.selected {
                    "row sel".to_owned()
                } else {
                    "row".to_owned()
                },
            })
            .collect()
    }

    fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.first_visible = self.first_visible.min(self.selected as f32);
    }

    fn select_down(&mut self) {
        let last = self.entries.len().saturating_sub(1);
        self.selected = (self.selected + 1).min(last);
    }

    /// Scrolls by `rows`, clamped to the listing (the wheel handler's target).
    fn scroll_rows(&mut self, rows: f32) {
        let last = self.entries.len().saturating_sub(1) as f32;
        self.first_visible = (self.first_visible + rows).clamp(0.0, last);
    }

    /// Keeps the cursor inside the window after a move or a resize.
    fn follow_cursor(&mut self, visible: usize) {
        let first = self.first_visible as usize;
        if self.selected >= first + visible {
            self.first_visible = (self.selected + 1 - visible) as f32;
        } else if self.selected < first {
            self.first_visible = self.selected as f32;
        }
    }

    /// Enters the selected directory; a file is reported rather than opened (the
    /// kit launches nothing by itself).
    fn open(&mut self) {
        let Some(entry) = self.entries.get(self.selected) else {
            return;
        };
        if entry.is_dir {
            let target = self.dir.join(&entry.name);
            self.load(target);
        } else {
            self.message = format!("{} — {}", entry.name, human_size(entry.size));
        }
    }

    /// Goes to the parent directory, if there is one.
    fn parent(&mut self) {
        if let Some(parent) = self.dir.parent().map(Path::to_path_buf) {
            self.load(parent);
        }
    }
}

/// Reads and sorts one directory. Unreadable entries are skipped rather than
/// failing the whole listing.
fn read_entries(dir: &Path) -> std::io::Result<Vec<Entry>> {
    let mut entries: Vec<Entry> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| {
            let metadata = entry.metadata().ok();
            Entry {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: metadata.as_ref().is_some_and(std::fs::Metadata::is_dir),
                size: metadata.as_ref().map_or(0, std::fs::Metadata::len),
            }
        })
        .collect();
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    Ok(entries)
}

/// Formats a byte count. Hand-rolled on purpose: the kit's locale-aware number
/// formatting (§7.2) is not built, so no grouping separator is available.
fn human_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * KB;
    const GB: u64 = MB * KB;
    if bytes < KB {
        format!("{bytes} B")
    } else if bytes < MB {
        format!("{} KB", bytes / KB)
    } else if bytes < GB {
        format!("{} MB", bytes / MB)
    } else {
        format!("{} GB", bytes / GB)
    }
}

/// Builds the whole UI from the current state. Every `{…}` below reads a real
/// local of this function — that is what the inline macro buys over a compiled
/// `.okui` file.
fn build_view(state: &mut State, frame: &FrameInfo) -> View {
    // How many rows fit: the list is what is left of the viewport under the toolbar
    // and above the status bar. One extra row so a partially-scrolled list has
    // something to show at the seam.
    let list_height = (frame.viewport.height - TOOLBAR_H - STATUS_H).max(0.0);
    let visible = ((list_height / ROW_H).ceil() as usize + 1).min(state.entries.len());
    state.follow_cursor(visible);

    let path = maybe(state.dir.display().to_string());
    let status = maybe(format!(
        "{} — {} of {} shown",
        state.message,
        visible,
        state.entries.len()
    ));
    let rows = state.rows(visible);
    okui! {
        <div class="app">
            <div class="toolbar">
                <div class="btn" on:click={goParent}><span class="btn-label">"Назад"</span></div>
                <div class="btn" on:click={openEntry}><span class="btn-label">"Открыть"</span></div>
                <div class="btn" on:click={selectUp}><span class="btn-label">"Выше"</span></div>
                <div class="btn" on:click={selectDown}><span class="btn-label">"Ниже"</span></div>
                <span class="path">{path}</span>
            </div>
            <div class="list">
                <For each={rows} as={row} key={row.key}>
                    <div class={row.class}>
                        <span class="kind">{row.kind}</span>
                        <span class="name">{row.name}</span>
                        <span class="size">{row.size}</span>
                    </div>
                </For>
            </div>
            <div class="status">
                <span class="hint">{status}</span>
            </div>
        </div>
    }
}

fn main() -> Result<(), obscur_kit::AppError> {
    // Start in $HOME, falling back to the working directory.
    let start = std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("/"));

    if blank_text() {
        println!("[glyph-probe] text blanked (4 static button labels remain)");
    }

    let state = Rc::new(RefCell::new(State {
        dir: PathBuf::new(),
        entries: Vec::new(),
        selected: 0,
        message: String::new(),
        first_visible: 0.0,
    }));
    state.borrow_mut().load(start);

    // `on:click=name` in the markup compiles to `fallback_symbol(name)`, so the
    // handler registered here under the same name is the one it reaches.
    let command = |name: &str| -> InternedSymbol { fallback_symbol(name) };

    let view_state = state.clone();
    let up = state.clone();
    let down = state.clone();
    let open = state.clone();
    let parent = state.clone();
    let scroller = state.clone();

    App::new("obscur-kit — file manager (.okui)")
        .app_id("obscur-kit-file-manager")
        .with_size(760.0, 560.0)
        .stylesheet(if std::env::var_os("OBSCUR_NO_BG").is_some() {
            println!("[overdraw-probe] backgrounds stripped");
            CSS_NO_BG
        } else {
            CSS
        })
        .view_with(move |frame| build_view(&mut view_state.borrow_mut(), frame))
        .on_command(command("selectUp"), move || up.borrow_mut().select_up())
        .on_command(command("selectDown"), move || {
            down.borrow_mut().select_down();
        })
        .on_command(command("openEntry"), move || open.borrow_mut().open())
        .on_command(command("goParent"), move || parent.borrow_mut().parent())
        // The kit no longer scrolls the list (it only ever holds the visible rows),
        // so the wheel arrives here and moves the app's own window.
        .on_scroll(move |dy| scroller.borrow_mut().scroll_rows(dy / ROW_H))
        .run()
}
