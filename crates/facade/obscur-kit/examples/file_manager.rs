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

use obscur_kit::App;
use obscur_kit::identity::ComponentId;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_ir::name_binding::fallback_symbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

/// The row height the model is estimated at, matching the sheet. The runtime sizes
/// its window and its spacers from this until real measurement refines it.
const ROW_H: f32 = 24.0;

/// The registered virtual list's name.
const FILES: InternedSymbol = InternedSymbol::from_raw(0x00F1_0001);

/// The styling. Authored CSS, installed through the motion-aware pipeline by
/// [`App::stylesheet`].
const CSS: &str = "\
.app      { background-color: rgba(18, 22, 31, 0.78); width: 100%; height: 100%; \
            display: flex; flex-direction: column; }\n\
.toolbar  { background-color: #1b2130; width: 100%; height: 46px; \
            display: flex; flex-direction: row; gap: 8px; }\n\
.btn      { background-color: #2b3348; width: 96px; height: 30px; }\n\
.btn:hover{ background-color: #3d4a6b; }\n\
.btn-label{ color: #e6e9f0; width: 100%; height: 20px; }\n\
.path     { color: #9fb0d0; width: 100%; height: 30px; }\n\
.list     { width: 100%; height: 100%; display: flex; flex-direction: column; \
            overflow: scroll; }\n\
.row      { width: 100%; height: 24px; display: flex; flex-direction: row; \
            gap: 10px; overflow: hidden; }\n\
.row:hover{ background-color: #1b2130; }\n\
.sel      { background-color: #2b3f66; }\n\
.kind     { color: #7f8db0; width: 52px; height: 20px; }\n\
.name     { color: #e6e9f0; width: 100%; height: 20px; \
            white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }\n\
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
.list     { width: 100%; height: 100%; display: flex; flex-direction: column; \
            overflow: scroll; }\n\
.row      { width: 100%; height: 24px; display: flex; flex-direction: row; \
            gap: 10px; overflow: hidden; }\n\
.kind     { color: #7f8db0; width: 52px; height: 20px; }\n\
.name     { color: #e6e9f0; width: 100%; height: 20px; \
            white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }\n\
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

/// The browser's state: where we are, what is there, and which line is selected.
struct State {
    dir: PathBuf,
    entries: Vec<Entry>,
    selected: usize,
    message: String,
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

    fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn select_down(&mut self) {
        let last = self.entries.len().saturating_sub(1);
        self.selected = (self.selected + 1).min(last);
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
    // Directories first, then by name — case-insensitively.
    //
    // A plain `str` comparison orders by code point, which puts every capital
    // Cyrillic letter (А-Я, U+0410..) before every lowercase one (а-я, U+0430..): in
    // a home directory that drops "образы" after "Шаблоны" instead of among the
    // о-names, which reads as broken. Lowercasing both sides is the pragmatic fix;
    // real locale-aware collation (§7.2) is not built in the kit, so this is the
    // app's call, and the raw name breaks ties so the order stays stable.
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
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
fn build_view(state: &State) -> View {
    let path = maybe(state.dir.display().to_string());
    let status = maybe(state.message.clone());
    // The list is one marker: the runtime windows it against the viewport and this
    // list's own scroll offset, and calls the registered builder for the visible
    // indices only. No row arithmetic here, and no `ROW_H` maths to keep in step
    // with the sheet beyond the row height the model is estimated at.
    let list = ViewNode::element(ComponentId::from_raw(0x00F0_0001))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("list".into())))
        .with_children(ViewFragment::from_nodes(vec![ViewNode::virtual_list(
            FILES,
            state.entries.len(),
            ROW_H,
        )]));
    let shell = okui! {
        <div class="app">
            <div class="toolbar">
                <div class="btn" on:click={goParent}><span class="btn-label">"Назад"</span></div>
                <div class="btn" on:click={openEntry}><span class="btn-label">"Открыть"</span></div>
                <div class="btn" on:click={selectUp}><span class="btn-label">"Выше"</span></div>
                <div class="btn" on:click={selectDown}><span class="btn-label">"Ниже"</span></div>
                <span class="path">{path}</span>
            </div>
            <div class="status">
                <span class="hint">{status}</span>
            </div>
        </div>
    };
    // Splice the virtual list between the toolbar and the status bar. The macro
    // cannot emit the marker itself yet — that is the remaining `<VirtualFor>` work
    // — so the shell is authored and the list is placed.
    splice_list(shell, list)
}

/// Inserts `list` as the second child of the shell's root.
fn splice_list(shell: View, list: ViewNode) -> View {
    let mut roots = shell.into_fragment().into_nodes();
    let ViewNode::Element { children, .. } = &mut roots[0] else {
        return View::from_fragment(ViewFragment::from_nodes(roots));
    };
    let mut kids = std::mem::take(children).into_nodes();
    kids.insert(1, list);
    *children = ViewFragment::from_nodes(kids);
    View::from_fragment(ViewFragment::from_nodes(roots))
}

/// Builds one row, by model index — the builder the runtime calls for the window.
///
/// This is where `okui!` earns its place: the bindings read the real entry.
fn row_node(state: &State, index: usize) -> ViewNode {
    let Some(entry) = state.entries.get(index) else {
        return ViewNode::element(ComponentId::from_raw(0x00F0_0002));
    };
    let name = maybe(entry.name.clone());
    let kind = maybe(if entry.is_dir { "DIR" } else { "file" }.to_owned());
    let size = maybe(if entry.is_dir {
        String::new()
    } else {
        human_size(entry.size)
    });
    let class = if index == state.selected {
        "row sel"
    } else {
        "row"
    };
    let row = okui! {
        <div class={class}>
            <span class="kind">{kind}</span>
            <span class="name">{name}</span>
            <span class="size">{size}</span>
        </div>
    };
    row.into_fragment()
        .into_nodes()
        .pop()
        .unwrap_or_else(|| ViewNode::element(ComponentId::from_raw(0x00F0_0002)))
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
    }));
    state.borrow_mut().load(start);

    // `on:click=name` in the markup compiles to `fallback_symbol(name)`, so the
    // handler registered here under the same name is the one it reaches.
    let command = |name: &str| -> InternedSymbol { fallback_symbol(name) };

    let view_state = state.clone();
    let row_state = state.clone();
    let up = state.clone();
    let down = state.clone();
    let open = state.clone();
    let parent = state.clone();

    App::new("obscur-kit — file manager (.okui)")
        .app_id("obscur-kit-file-manager")
        .with_size(760.0, 560.0)
        .stylesheet(if std::env::var_os("OBSCUR_NO_BG").is_some() {
            println!("[overdraw-probe] backgrounds stripped");
            CSS_NO_BG
        } else {
            CSS
        })
        .view(move || build_view(&view_state.borrow()))
        // The runtime calls this only for the indices the viewport shows.
        .virtual_list(FILES, move |index| row_node(&row_state.borrow(), index))
        .on_command(command("selectUp"), move || up.borrow_mut().select_up())
        .on_command(command("selectDown"), move || {
            down.borrow_mut().select_down();
        })
        .on_command(command("openEntry"), move || open.borrow_mut().open())
        .on_command(command("goParent"), move || parent.borrow_mut().parent())
        .run()
}
