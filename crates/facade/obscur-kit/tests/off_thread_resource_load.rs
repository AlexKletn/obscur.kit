#![doc = include_str!("off_thread_resource_load.doc/_module.md")]
#![forbid(unsafe_code)]
#![cfg(all(
    feature = "resource-fs",
    feature = "resource-decode",
    feature = "task-smol"
))]

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::ThreadId;

use obscur_kit::bootstrap::KitBootstrap;
use obscur_kit::css::compiler::StylesheetCompiler;
use obscur_kit::css::parser::StylesheetParser;
use obscur_kit::css::source::CssSource;
use obscur_kit::geometry::Size;
use obscur_kit::identity::{ComponentId, RuntimeId};
use obscur_kit::resources::port::{LoadedBytes, ResourceLoader};
use obscur_kit::resources::source::ResourceSource;
use obscur_kit::resources::state::ResourceError;
use obscur_kit::runtime::UiRuntime;
use obscur_kit::scene::command::PaintCommand;
use obscur_kit::scene::scene::SceneSnapshot;
use obscur_kit::style::matching::Stylesheet;
use obscur_kit::view::view::{CLASS, View, ViewFragment, ViewNode, ViewProperty, ViewValue};

#[doc = include_str!("off_thread_resource_load.doc/VIEWPORT.md")]
const VIEWPORT: f32 = 64.0;

#[doc = include_str!("off_thread_resource_load.doc/RED_2X2_PNG.md")]
const RED_2X2_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0,
    0, 0, 253, 212, 154, 115, 0, 0, 0, 16, 73, 68, 65, 84, 120, 218, 99, 248, 207, 192, 0, 68, 12,
    16, 10, 0, 31, 238, 3, 253, 99, 94, 187, 91, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[doc = include_str!("off_thread_resource_load.doc/COUNTER.md")]
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[doc = include_str!("off_thread_resource_load.doc/temp_asset_dir.md")]
fn temp_asset_dir() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("obscur-kit-offthread-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("logo.png"), RED_2X2_PNG).expect("write png");
    dir
}

#[doc = include_str!("off_thread_resource_load.doc/ThreadRecordingLoader.md")]
struct ThreadRecordingLoader {
    inner: obscur_kit::resource_fs::loader::FsResourceLoader,
    loaded_on: Mutex<Vec<ThreadId>>,
}

impl ThreadRecordingLoader {
    fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            inner: obscur_kit::resource_fs::loader::FsResourceLoader::with_root(root),
            loaded_on: Mutex::new(Vec::new()),
        }
    }

    #[doc = include_str!("off_thread_resource_load.doc/loaded_on.md")]
    fn loaded_on(&self) -> Vec<ThreadId> {
        self.loaded_on.lock().expect("lock").clone()
    }
}

impl ResourceLoader for ThreadRecordingLoader {
    fn can_load(&self, source: &ResourceSource) -> bool {
        self.inner.can_load(source)
    }

    fn load(&self, source: &ResourceSource) -> Result<LoadedBytes, ResourceError> {
        self.loaded_on
            .lock()
            .expect("lock")
            .push(std::thread::current().id());
        self.inner.load(source)
    }
}

#[doc = include_str!("off_thread_resource_load.doc/compile.md")]
fn compile(css: &str) -> Stylesheet {
    let src = CssSource::anonymous(css);
    let parsed = StylesheetParser::new(&src).parse();
    StylesheetCompiler::new().compile(&parsed).matchable
}

#[doc = include_str!("off_thread_resource_load.doc/img_view.md")]
fn img_view() -> View {
    let node = ViewNode::element(ComponentId::from_raw(40))
        .with_prop(ViewProperty::new(CLASS, ViewValue::Text("img".into())));
    View::from_fragment(ViewFragment::from_nodes(vec![node]))
}

fn img_sheet() -> Stylesheet {
    compile(r#".img { background-image: url("logo.png"); width: 40px; height: 30px }"#)
}

fn draw_image_count(snapshot: &SceneSnapshot) -> usize {
    snapshot
        .display_list()
        .commands()
        .iter()
        .filter(|c| matches!(c, PaintCommand::DrawImage { .. }))
        .count()
}

#[doc = include_str!("off_thread_resource_load.doc/wait_for_loads.md")]
fn wait_for_loads(loader: &ThreadRecordingLoader, n: usize) {
    for _ in 0..500 {
        if loader.loaded_on().len() >= n {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[doc = include_str!("off_thread_resource_load.doc/an_installed_blocking_executor_runs_the_load_on_another_thread.md")]
#[test]
fn an_installed_blocking_executor_runs_the_load_on_another_thread() {
    let dir = temp_asset_dir();
    let loader = std::sync::Arc::new(ThreadRecordingLoader::new(&dir));
    let sheet = img_sheet();

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    runtime.set_resource_providers(
        loader.clone(),
        std::sync::Arc::new(obscur_kit::resource_decode::decoder::ImageDecoder::new()),
    );
    // The one call under test.
    let _spawner = KitBootstrap::new().install_blocking_executor(&mut runtime);

    let here = std::thread::current().id();
    let first = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));

    // The frame handed the load off instead of doing it: nothing to paint yet.
    assert_eq!(
        draw_image_count(first.snapshot()),
        0,
        "the frame must not block on the load"
    );

    wait_for_loads(&loader, 1);
    let threads = loader.loaded_on();
    assert_eq!(threads.len(), 1, "the load ran exactly once");
    assert_ne!(
        threads[0], here,
        "§1.5: the load must run OFF the frame thread, not on it"
    );

    // And the typed result comes back and is committed, so the image paints.
    //
    // `poll_resource_tasks` is deliberately NOT called: the claim under test is
    // that `run_frame` commits results at its own synchronization point, so a
    // host that renders needs no polling discipline. Rendering in a bounded loop
    // waits for the *pool* (which decides when the decode finishes and the
    // message is sent), not for the commit — the commit is what is being proved,
    // and nothing but the frame path can perform it here.
    let painted = (0..500).any(|_| {
        let frame = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
        if draw_image_count(frame.snapshot()) == 1 {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        false
    });
    assert!(
        painted,
        "the off-thread result must be committed at the frame's synchronization \
         point, with no poll_resource_tasks call"
    );
}

#[doc = include_str!("off_thread_resource_load.doc/with_no_executor_the_load_runs_on_the_frame_thread.md")]
#[test]
fn with_no_executor_the_load_runs_on_the_frame_thread() {
    let dir = temp_asset_dir();
    let loader = std::sync::Arc::new(ThreadRecordingLoader::new(&dir));
    let sheet = img_sheet();

    let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(VIEWPORT, VIEWPORT));
    runtime.set_resource_providers(
        loader.clone(),
        std::sync::Arc::new(obscur_kit::resource_decode::decoder::ImageDecoder::new()),
    );

    let here = std::thread::current().id();
    let first = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));

    // The frame phase must stay short: the load is deferred, so the loader was not
    // reached and nothing paints yet.
    assert_eq!(
        draw_image_count(first.snapshot()),
        0,
        "the deferred load has not run during the frame phase"
    );
    assert!(
        loader.loaded_on().is_empty(),
        "the loader is never invoked inside render_now"
    );

    // Poll runs the deferred load — on THIS thread, with no executor and so no
    // thread ever spawned — and the next frame composes the decoded image.
    let applied = runtime.poll_resource_tasks();
    assert_eq!(applied, 1, "the deferred load is run and committed on poll");
    let threads = loader.loaded_on();
    assert_eq!(threads.len(), 1, "the load ran exactly once");
    assert_eq!(
        threads[0], here,
        "with no executor the load runs on the frame thread — no thread spawned"
    );

    let second = runtime.render_now(&img_view(), std::slice::from_ref(&sheet));
    assert_eq!(
        draw_image_count(second.snapshot()),
        1,
        "the re-render composes the now-decoded image",
    );
}

#[doc = include_str!("off_thread_resource_load.doc/a_finished_load_is_committed_by_the_next_frame_with_no_polling.md")]
#[test]
fn a_finished_load_is_committed_by_the_next_frame_with_no_polling() {
    let dir = temp_asset_dir();
    let loader = std::sync::Arc::new(ThreadRecordingLoader::new(&dir));
    let sheet = img_sheet();

    let mut host = KitBootstrap::new().headless_host();
    host.runtime_mut().set_resource_providers(
        loader.clone(),
        std::sync::Arc::new(obscur_kit::resource_decode::decoder::ImageDecoder::new()),
    );

    let first = host.render_styled(&img_view(), std::slice::from_ref(&sheet));
    assert_eq!(
        draw_image_count(first.snapshot()),
        0,
        "the load was handed off"
    );

    // Run the job (not `run_tasks`, which would also poll): the result is now a
    // typed message sitting on the runtime's channel, uncommitted.
    host.executor().run();

    let second = host.render_styled(&img_view(), std::slice::from_ref(&sheet));
    assert_eq!(
        draw_image_count(second.snapshot()),
        1,
        "the frame must commit the waiting result itself"
    );
}

#[doc = include_str!("off_thread_resource_load.doc/the_headless_host_still_loads_on_the_calling_thread.md")]
#[test]
fn the_headless_host_still_loads_on_the_calling_thread() {
    let dir = temp_asset_dir();
    let loader = std::sync::Arc::new(ThreadRecordingLoader::new(&dir));
    let sheet = img_sheet();

    let mut host = KitBootstrap::new().headless_host();
    host.runtime_mut().set_resource_providers(
        loader.clone(),
        std::sync::Arc::new(obscur_kit::resource_decode::decoder::ImageDecoder::new()),
    );

    let here = std::thread::current().id();
    host.render_styled(&img_view(), std::slice::from_ref(&sheet));

    // Determinism means the job is queued, not running: nothing has touched the
    // loader yet, and no amount of waiting would change that.
    assert!(
        loader.loaded_on().is_empty(),
        "the deterministic executor must not start the load until told"
    );

    let applied = host.run_tasks();
    assert_eq!(applied, 1, "run_tasks is what makes the load happen");
    let threads = loader.loaded_on();
    assert_eq!(threads.len(), 1);
    assert_eq!(
        threads[0], here,
        "§7.4: the deterministic executor runs the load INLINE on the caller"
    );
}
