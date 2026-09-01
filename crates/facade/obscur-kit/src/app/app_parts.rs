#![doc = include_str!("app_parts.doc/_module.md")]

use crate::bootstrap::{CssPipeline, FontSet, KitBootstrap};
use crate::control::AppControl;
use crate::geometry::Size;
use crate::identity::{InternedSymbol, RuntimeId};
use crate::interaction::event::UiEvent;
use crate::runtime::{CommandHandler, UiRuntime};
use crate::view::view::{View, ViewNode};

#[doc = include_str!("app_parts.doc/RowBuilder.md")]
pub(super) type RowBuilder = Box<dyn FnMut(usize) -> ViewNode + 'static>;

#[doc = include_str!("app_parts.doc/RowClick.md")]
pub(super) type RowClick = Box<dyn FnMut(usize, u32) + 'static>;

#[doc = include_str!("app_parts.doc/RowHold.md")]
pub(super) type RowHold = Box<dyn FnMut(usize) + 'static>;

#[doc = include_str!("app_parts.doc/ScrollHandler.md")]
pub(super) type ScrollHandler = Box<dyn FnMut(f32) + 'static>;

#[doc = include_str!("app_parts.doc/DiagnosticSinkBox.md")]
pub(super) type DiagnosticSinkBox = Box<dyn obscur_diagnostics::DiagnosticSink + 'static>;

#[doc = include_str!("app_parts.doc/AppParts.md")]
pub(super) struct AppParts {
    // A desktop surface has no title and asks for no size — the platform decides
    // its extent, and it has no place in the window stack to be labelled in. So in
    // a build with no windowing backend these two are genuinely unread, and saying
    // so here is better than giving the layer runner a use for them it does not have.
    #[cfg_attr(not(feature = "window-winit"), allow(dead_code))]
    pub(super) title: String,
    pub(super) app_id: String,
    #[cfg_attr(not(feature = "window-winit"), allow(dead_code))]
    pub(super) size: (f32, f32),
    pub(super) stylesheets: Vec<String>,
    #[doc = include_str!("app_parts.doc/fonts.md")]
    pub(super) fonts: FontSet,
    pub(super) commands: Vec<(InternedSymbol, CommandHandler)>,
    pub(super) control: AppControl,
    pub(super) scroll: Option<ScrollHandler>,
    pub(super) virtual_lists: Vec<(InternedSymbol, RowBuilder)>,
    pub(super) virtual_clicks: Vec<(InternedSymbol, RowClick)>,
    pub(super) virtual_holds: Vec<(InternedSymbol, RowHold)>,
    #[doc = include_str!("app_parts.doc/diagnostics.md")]
    pub(super) diagnostics: Option<DiagnosticSinkBox>,
}

#[doc = include_str!("app_parts.doc/BootedApp.md")]
pub(super) struct BootedApp {
    pub(super) runtime: UiRuntime,
    pub(super) css: CssPipeline,
    #[doc = include_str!("app_parts.doc/boot.md")]
    pub(super) boot: KitBootstrap,
    pub(super) control: AppControl,
    pub(super) scroll: Option<ScrollHandler>,
}

impl AppParts {
    #[doc = include_str!("app_parts.doc/boot_fn.md")]
    pub(super) fn boot(
        self,
        builder: Box<dyn FnMut() -> View + 'static>,
        viewport: Size,
    ) -> BootedApp {
        let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), viewport);

        // Before anything else, because everything below reports: the stylesheet
        // compile is the loudest producer in the boot and its complaints are the
        // ones an application most needs (audit 3-C).
        install_diagnostics(&mut runtime, self.diagnostics);

        // The root view builder and command handlers are the app's own; forward them
        // into the runtime verbatim (the same shapes `set_root_view` / `on_command`
        // take). A `Box<dyn FnMut() -> View>` is itself `FnMut() -> View`.
        runtime.set_root_view(builder);
        for (command, mut handler) in self.commands {
            // The handler was already normalized by the builder (an app may register a
            // zero-argument closure or an event-taking one, and `App::on_command` folded
            // both into this one shape), so what is forwarded is a trampoline that hands
            // the propagating event straight through.
            runtime.on_command(command, move |event: &mut UiEvent| handler.call(event));
        }
        for (id, build) in self.virtual_lists {
            runtime.register_virtual_list(id, build);
        }
        for (id, handler) in self.virtual_clicks {
            runtime.on_virtual_row_click(id, handler);
        }
        for (id, handler) in self.virtual_holds {
            runtime.on_virtual_row_long_press(id, handler);
        }

        // The application's own faces travel with the bootstrap rather than
        // being installed here: a face has to reach the *renderer* under the
        // same handle it reaches the resolver under, and only the runner that
        // owns a renderer can close that loop.
        let mut boot = KitBootstrap::new();
        for face in self.fonts.faces() {
            boot = boot.with_font(face.clone());
        }

        // Defaults-on that need no surface (§5.18 / §3.3): the UA default sheet (below
        // every author sheet), the accumulated author CSS through the motion-aware
        // pipeline, filesystem images, and the threaded blocking executor (correct for
        // a standalone process — neither of these runners is a headless host).
        boot.install_default_stylesheet(&mut runtime);
        // …and the controls that sheet styles, mounted rather than merely named: the id
        // an authored `<checkbox>` carries is only a handle until a factory sits under
        // it. The only failure is a duplicate id, which cannot happen on a runtime this
        // function has just created.
        let _ = boot.install_builtin_components(&mut runtime);
        let mut css = CssPipeline::new();
        // The authored set goes in through the same handle a later replacement uses, so
        // the identity and revision each source is stamped with (§4.12's cache key) is
        // decided in exactly one place — and the first install is not a special case.
        // Queued last, so `App::stylesheet` wins over a replacement an application asked
        // for before `run`: those sheets are the initial set by definition, and the
        // application's own replacement belongs to the loop that has not started yet.
        self.control.set_stylesheets(self.stylesheets);
        self.control.apply(&mut runtime, &mut css);
        install_resources(&boot, &mut runtime);

        BootedApp {
            runtime,
            css,
            boot,
            control: self.control,
            scroll: self.scroll,
        }
    }
}

#[doc = include_str!("app_parts.doc/install_diagnostics.md")]
fn install_diagnostics(runtime: &mut UiRuntime, chosen: Option<DiagnosticSinkBox>) {
    if let Some(sink) = chosen {
        runtime.set_diagnostic_sink(sink);
        return;
    }
    // No sink chosen. The runtime's own default discards everything, which is
    // right for a library and wrong for a standalone process: a toolkit that
    // reports carefully into a sink nobody installed is indistinguishable from
    // one that does not report at all (audit 3-C). In a debug build, say it.
    #[cfg(debug_assertions)]
    runtime.set_diagnostic_sink(Box::new(crate::bootstrap::StderrDiagnostics::new()));
}

#[doc = include_str!("app_parts.doc/install_resources.md")]
#[allow(unused_variables)]
fn install_resources(boot: &KitBootstrap, runtime: &mut UiRuntime) {
    #[cfg(all(feature = "resource-fs", feature = "resource-decode"))]
    {
        boot.install_image_resources(runtime, ".");
        boot.install_vector_resources(runtime);
    }
    #[cfg(feature = "task-smol")]
    {
        let _spawner = boot.install_blocking_executor(runtime);
    }
}
