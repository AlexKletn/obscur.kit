//! **SPECIFICATION-DERIVED** font-provider and cache cases for obscur.kit §5.18
//! ("Fonts"), §5.19 ("Caches") and §5.20 ("Persistent derived cache").
//!
//! Every expected value below was produced by an agent reading obscur.kit §5 with
//! **no access to this repository's source**.
#![forbid(unsafe_code)]
#![cfg(feature = "render-cpu")]

#[path = "spec_ch5_support/mod.rs"]
mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use obscur_kit::scene::command::PaintCommand;
use obscur_kit::text::font::{
    FontAttributes, FontFace, FontFamily, FontId, FontMetrics, FontResolveRequest, FontResolver,
    FontSource, ResolvedFont,
};

use support::{commands, compile, describe, el, runtime, txt, view};

/// The viewport every case renders under.
const VW: f32 = 400.0;

/// A [`FontResolver`] that always answers with one face and counts its calls.
struct OneFace {
    id: FontId,
    face: FontFace,
    calls: Arc<AtomicUsize>,
}

impl FontResolver for OneFace {
    fn resolve(&self, _request: &FontResolveRequest) -> Option<ResolvedFont> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Some(ResolvedFont::new(self.id, self.face.clone()))
    }
}

/// A named face with idealized metrics.
fn face(name: &str) -> FontFace {
    FontFace::new(
        FontFamily::named(name),
        FontAttributes::default(),
        FontSource::system(name),
        FontMetrics::new(1000, 800.0, -200.0, 0.0),
    )
}

/// Every `DrawText` run's font handle in the last frame, in paint order.
fn run_fonts(rt: &obscur_kit::runtime::UiRuntime) -> Vec<u32> {
    commands(rt)
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.font()),
            _ => None,
        })
        .collect()
}

/// §5.18 "Default font provider: - system fonts on native platforms; - bundled
/// fallback set for WASM/isolated environments; … - usable rendering out of the
/// box."
///
/// The isolated case: system font enumeration returns nothing. Text must still
/// render, from the bundled set.
#[cfg(all(feature = "text-harfrust", feature = "text-unicode"))]
#[test]
fn ft_01_the_default_provider_renders_in_an_isolated_environment() {
    use obscur_kit::bootstrap::{FontSet, KitBootstrap};

    let mut rt = runtime(VW, VW);
    // An EMPTY font set is the WASM/isolated environment: nothing enumerated.
    let _ = KitBootstrap::new().install_text_stack(&mut rt, &FontSet::new());

    let sheet = compile(".t{position:absolute;left:0;top:0;font-size:16px}");
    let v = view(vec![el(10, "t", vec![txt("Hello")])]);
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));

    let list = commands(&rt);
    let texts: Vec<f32> = list
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.advance()),
            _ => None,
        })
        .collect();

    assert!(
        !texts.is_empty(),
        "at least one glyph run is produced from the bundled fallback set; \
         the display list is {:?}",
        list.iter().map(describe).collect::<Vec<_>>()
    );
    assert!(
        texts.iter().sum::<f32>() > 0.0,
        "the run's total advance is greater than zero — the text is not invisible"
    );
    let notdefs = list
        .iter()
        .filter_map(|c| match c {
            PaintCommand::DrawText { run, .. } => Some(run.glyphs()),
            _ => None,
        })
        .flatten()
        .filter(|g| g.glyph_id() == 0)
        .count();
    assert_eq!(notdefs, 0, "zero .notdef / missing glyphs");
}

/// §5.18 "The provider is replaceable." With §5.19 "- glyphs;" and
/// "- correctness transparency;".
///
/// The failure this pins is a real one the suite already suffered: a font stack
/// can be swapped out and the tests keep passing because shaped runs are served
/// from caches keyed on text and size but **not** on the provider identity.
#[test]
fn ft_02_replacing_the_provider_invalidates_the_shaping_caches() {
    let sheet = compile(".t{position:absolute;left:0;top:0;font-size:16px}");
    let v = view(vec![el(10, "t", vec![txt("Hello")])]);

    let original_calls = Arc::new(AtomicUsize::new(0));
    let mut rt = runtime(VW, VW);
    rt.set_font_resolver(Box::new(OneFace {
        id: FontId::new(7),
        face: face("Original"),
        calls: Arc::clone(&original_calls),
    }));
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));
    assert_eq!(
        run_fonts(&rt),
        vec![7],
        "the first frame shapes with the original provider"
    );

    // Replace the provider; the very NEXT frame must use the custom face.
    let calls_before = original_calls.load(Ordering::Relaxed);
    rt.set_font_resolver(Box::new(OneFace {
        id: FontId::new(42),
        face: face("Custom"),
        calls: Arc::new(AtomicUsize::new(0)),
    }));
    let _ = rt.render_now(&v, std::slice::from_ref(&sheet));

    assert_eq!(
        run_fonts(&rt),
        vec![42],
        "every run's face is the custom one in the very NEXT frame, not after \
         a resize or an unrelated invalidation"
    );
    assert_eq!(
        original_calls.load(Ordering::Relaxed),
        calls_before,
        "the replaced provider is queried 0 times after replacement"
    );
}

/// §5.19 "Requirements: … - correctness transparency;".
///
/// A cache keyed on too little state leaves stale paint behind. The
/// mutate-and-restore cycle is exactly the sequence a straight-line test never
/// performs.
#[test]
fn ca_01_a_mutate_and_restore_cycle_reproduces_the_original_scene() {
    // A tree exercising text, a gradient and a rounded clip.
    let base = ".c{position:absolute;left:0;top:0;width:120px;height:120px;\
                   border-radius:8px;overflow:hidden;\
                   background-image:linear-gradient(rgb(255,0,0),rgb(0,0,255))}\
                .r{width:60px;height:60px;border-radius:12px;\
                   background-color:rgb(0,255,0);font-size:16px}";
    let v = view(vec![el(10, "c", vec![el(11, "r", vec![txt("hello")])])]);

    // (a) a fresh runtime, one frame.
    let mut fresh = runtime(VW, VW);
    let sheet = compile(base);
    let _ = fresh.render_now(&v, std::slice::from_ref(&sheet));
    let a = commands(&fresh);

    // (c) the same runtime, after 10 intervening frames that mutate the tree's
    //     styles and geometry and then restore them exactly.
    let mut cycled = runtime(VW, VW);
    let _ = cycled.render_now(&v, std::slice::from_ref(&sheet));
    for step in 0..10 {
        let mutated = compile(&format!(
            ".c{{position:absolute;left:{}px;top:0;width:{}px;height:120px;\
                border-radius:{}px;overflow:hidden;\
                background-image:linear-gradient(rgb(0,255,0),rgb(255,0,0))}}\
             .r{{width:{}px;height:60px;border-radius:4px;\
                background-color:rgb(0,0,255);font-size:{}px}}",
            step,
            100 + step,
            step,
            40 + step,
            12 + step
        ));
        let _ = cycled.render_now(&v, std::slice::from_ref(&mutated));
    }
    let restored = compile(base);
    let _ = cycled.render_now(&v, std::slice::from_ref(&restored));
    let c = commands(&cycled);

    assert_eq!(
        a.len(),
        c.len(),
        "restoring the styles restores the command count: {:?} vs {:?}",
        a.iter().map(describe).collect::<Vec<_>>(),
        c.iter().map(describe).collect::<Vec<_>>()
    );
    for (i, (left, right)) in a.iter().zip(c.iter()).enumerate() {
        assert!(
            left == right,
            "command {i} differs with zero tolerance after a mutate-and-restore \
             cycle: {} vs {}",
            describe(left),
            describe(right)
        );
    }
}

/// §5.20 "Requirements: - versioned; - content-addressed where useful;
/// - compatibility-aware; - disposable; - safe to delete."
#[cfg(feature = "cache-fs")]
#[test]
fn ca_03_the_persistent_store_is_disposable_and_version_gated() {
    use obscur_kit::cache_fs::store::FsCacheStore;
    use obscur_kit::identity::ResourceId;
    use obscur_kit::resources::cache::CacheKey;
    use obscur_kit::resources::port::CacheStore;
    use obscur_kit::resources::state::ResourceVersion;

    let n = std::process::id();
    let root = std::env::temp_dir().join(format!("obscur-kit-ch5-store-{n}"));
    let _ = std::fs::remove_dir_all(&root);

    let key = CacheKey::for_resource(ResourceId::from_raw(1));
    let v1 = ResourceVersion::initial();
    let v2 = v1.next();
    let payload: Arc<[u8]> = Arc::from(vec![1_u8, 2, 3, 4]);

    // Run 1 — populate.
    {
        let mut store = FsCacheStore::new(&root);
        store.put(key, v1, Arc::clone(&payload));
        assert_eq!(
            store.get(&key, v1).as_deref(),
            Some(&*payload),
            "run 1 reads back what it wrote"
        );
    }

    // Run 2 — the whole store directory is deleted before start.
    {
        std::fs::remove_dir_all(&root).expect("the store is safe to delete");
        let store = FsCacheStore::new(&root);
        assert!(
            store.get(&key, v1).is_none(),
            "a deleted store starts empty rather than failing"
        );
    }

    // Run 3 — the store survives but the version stamp changes.
    {
        let mut store = FsCacheStore::new(&root);
        store.put(key, v1, Arc::clone(&payload));
        let store = FsCacheStore::new(&root);
        assert!(
            store.get(&key, v2).is_none(),
            "an entry derived under an incompatible version is IGNORED, not \
             deserialized into a plausible but wrong shape"
        );
    }

    // Run 4 — the store survives but one entry's bytes are corrupt.
    {
        let mut store = FsCacheStore::new(&root);
        store.put(key, v1, Arc::clone(&payload));
        // Overwrite every file under the root with garbage.
        corrupt_every_file(&root);
        let store = FsCacheStore::new(&root);
        let served = store.get(&key, v1);
        assert!(
            served.as_deref() != Some(&*payload),
            "a corrupt entry is not served as if it were intact"
        );
        // …and asking for it does not panic. Reaching here is the assertion.
    }

    let _ = std::fs::remove_dir_all(&root);
}

/// Overwrites the bytes of every regular file under `root`.
#[cfg(feature = "cache-fs")]
fn corrupt_every_file(root: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            corrupt_every_file(&path);
        } else {
            let _ = std::fs::write(&path, b"\xde\xad\xbe\xef corrupt");
        }
    }
}
