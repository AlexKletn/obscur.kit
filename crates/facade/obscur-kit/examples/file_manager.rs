#![doc = include_str!("file_manager.doc/_module.md")]
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use obscur_kit::App;
use obscur_kit::identity::ComponentId;
use obscur_kit::identity::InternedSymbol;
use obscur_kit::ui_macros::okui;
use obscur_kit::view::view::{View, ViewFragment, ViewNode};

#[doc = include_str!("file_manager.doc/ROW_H.md")]
const ROW_H: f32 = 24.0;

#[doc = include_str!("file_manager.doc/FILES.md")]
const FILES: InternedSymbol = InternedSymbol::named("files");

#[doc = include_str!("file_manager.doc/CSS.md")]
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

#[doc = include_str!("file_manager.doc/CSS_NO_BG.md")]
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

#[doc = include_str!("file_manager.doc/blank_text.md")]
fn blank_text() -> bool {
    std::env::var_os("OBSCUR_NO_TEXT").is_some()
}

#[doc = include_str!("file_manager.doc/maybe.md")]
fn maybe(text: String) -> String {
    if blank_text() { String::new() } else { text }
}

#[doc = include_str!("file_manager.doc/State.md")]
struct State {
    dir: PathBuf,
    entries: Vec<Entry>,
    selected: usize,
    message: String,
    #[doc = include_str!("file_manager.doc/generation.md")]
    generation: u64,
}

#[doc = include_str!("file_manager.doc/Entry.md")]
struct Entry {
    name: String,
    is_dir: bool,
    size: u64,
}

impl State {
    #[doc = include_str!("file_manager.doc/load.md")]
    fn load(&mut self, dir: PathBuf) {
        match read_entries(&dir) {
            Ok(entries) => {
                self.dir = dir;
                self.entries = entries;
                self.selected = 0;
                // A new listing is new content: bump the token so the list resets to
                // the top rather than opening at the previous directory's offset.
                self.generation += 1;
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

    #[doc = include_str!("file_manager.doc/select.md")]
    fn select(&mut self, index: usize) {
        let last = self.entries.len().saturating_sub(1);
        self.selected = index.min(last);
    }

    fn select_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn select_down(&mut self) {
        let last = self.entries.len().saturating_sub(1);
        self.selected = (self.selected + 1).min(last);
    }

    #[doc = include_str!("file_manager.doc/open.md")]
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

    #[doc = include_str!("file_manager.doc/describe.md")]
    fn describe(&mut self, index: usize) {
        self.select(index);
        let Some(entry) = self.entries.get(self.selected) else {
            return;
        };
        self.message = if entry.is_dir {
            format!("{} — directory (held)", entry.name)
        } else {
            format!("{} — {} (held)", entry.name, human_size(entry.size))
        };
    }

    #[doc = include_str!("file_manager.doc/parent.md")]
    fn parent(&mut self) {
        if let Some(parent) = self.dir.parent().map(Path::to_path_buf) {
            self.load(parent);
        }
    }
}

#[doc = include_str!("file_manager.doc/read_entries.md")]
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

#[doc = include_str!("file_manager.doc/human_size.md")]
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

#[doc = include_str!("file_manager.doc/build_view.md")]
fn build_view(state: &State) -> View {
    let path = maybe(state.dir.display().to_string());
    let status = maybe(state.message.clone());
    // The list is one marker: the runtime windows it against the viewport and this
    // list's own scroll offset, and calls the registered builder for the visible
    // indices only. No row arithmetic here, and no `ROW_H` maths to keep in step
    // with the sheet beyond the row height the model is estimated at.
    let list = ViewNode::element(ComponentId::from_raw(0x00F0_0001))
        .class("list")
        .with_children(ViewFragment::from_nodes(vec![
            ViewNode::virtual_list(FILES, state.entries.len(), ROW_H)
                .with_virtual_scroll_reset(state.generation),
        ]));
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
    // *can* emit a marker now — a `<VirtualFor>` lowers to one — but its row
    // template is emitted once with no item binding, so every realized row would be
    // identical. These rows are a function of the entry at a model index, which is
    // what the registered `virtual_list` builder is handed, so the shell is authored
    // and the list is placed.
    splice_list(shell, list)
}

#[doc = include_str!("file_manager.doc/splice_list.md")]
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

#[doc = include_str!("file_manager.doc/row_node.md")]
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
    row.into_single_node()
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
        generation: 0,
    }));
    state.borrow_mut().load(start);

    // `on:click={name}` in the markup lowers to the same named symbol — the brace
    // holds a command name, not an expression — so the handler registered here under
    // that name is the one it reaches.
    let command = InternedSymbol::named;

    let view_state = state.clone();
    let row_state = state.clone();
    let up = state.clone();
    let down = state.clone();
    let open = state.clone();
    let parent = state.clone();
    let click = state.clone();
    let hold = state.clone();

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
        // A row click selects it; a double click opens it (a directory is entered, a
        // file reported). The index is the entry's place in the whole listing — the
        // runtime resolves it from the click's geometry, which is the only thing that
        // can, since every row the builder emits shares one command symbol.
        .on_row_click(FILES, move |index, clicks| {
            let mut state = click.borrow_mut();
            state.select(index);
            if clicks >= 2 {
                state.open();
            }
        })
        // Holding a row is what a right click would be: a touchscreen has one
        // button and no modifiers, so this is the only route to a per-entry
        // action. It fires while the finger is still down, and the lift that
        // follows does not also select — the runtime consumes it.
        .on_row_long_press(FILES, move |index| hold.borrow_mut().describe(index))
        .run()
}
