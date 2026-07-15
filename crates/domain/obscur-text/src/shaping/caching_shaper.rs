//! A backend-agnostic caching decorator for the [`TextShaper`] port (spec §5.10).

use std::cell::RefCell;
use std::collections::HashMap;

use crate::font::{FontAttributes, FontFamily};
use crate::shaping::face_source_key::FaceSourceKey;
use crate::shaping::{GlyphRun, LineMetrics, ShapingRequest, TextDirection, TextShaper};

/// The full cache key: everything a [`ShapingRequest`] carries that can change
/// the shaped output.
///
/// The `f32` font size is stored via [`f32::to_bits`] so the key can derive
/// `Eq` and `Hash` (a raw `f32` cannot). Face identity is captured by family,
/// attributes, and the cheap [`FaceSourceKey`]; text, direction, and the
/// optional language hint round it out. Two requests with the same key produce
/// byte-identical shaped runs, and any observable difference lands in a
/// different bucket.
#[derive(Clone, PartialEq, Eq, Hash)]
struct ShapeKey {
    text: Box<str>,
    font_size_bits: u32,
    direction: TextDirection,
    language: Option<Box<str>>,
    family: FontFamily,
    attributes: FontAttributes,
    source: FaceSourceKey,
}

impl ShapeKey {
    /// Extracts a key from a [`ShapingRequest`], cloning only cheap owned data
    /// (never the font bytes).
    fn from_request(request: &ShapingRequest) -> Self {
        let face = request.face();
        Self {
            text: request.text().into(),
            font_size_bits: request.font_size().to_bits(),
            direction: request.direction(),
            language: request.language().map(Into::into),
            family: face.family().clone(),
            attributes: *face.attributes(),
            source: FaceSourceKey::from_source(face.source()),
        }
    }
}

/// A caching decorator that memoizes [`shape`](TextShaper::shape) results for an
/// inner [`TextShaper`] (spec §5.10).
///
/// The runtime shapes each text node up to three times per frame — once to
/// **measure**, once to **arrange**, and once to **compose** — and re-shapes
/// every frame even when nothing changed. Wrapping the active shaper in a
/// `CachingShaper` collapses all of that to a **single** call to the inner
/// engine per unique request: the first `shape` for a given text/size/face
/// invokes the inner shaper and stores the [`GlyphRun`]; every later identical
/// request (this frame or a future one) returns a clone of the cached run. The
/// output is byte-identical to shaping without the cache — this is a pure
/// performance change.
///
/// [`can_shape`](TextShaper::can_shape) and [`line_metrics`](TextShaper::line_metrics)
/// delegate straight to the inner shaper (both are cheap; only shaping is
/// cached).
///
/// The cache is single-threaded interior state (`RefCell`): the runtime shapes
/// on one thread, and `Box<dyn TextShaper>` carries no `Send`/`Sync` bound.
///
/// # Bound
///
/// The cache holds at most [`MAX_ENTRIES`](Self::MAX_ENTRIES) runs. When a new
/// entry would exceed the cap the whole map is cleared before inserting — a
/// simple, deterministic, occasional flush that keeps memory bounded without an
/// eviction-order dependency (headless output never observes iteration order,
/// only the single returned value).
pub struct CachingShaper {
    /// The wrapped shaper doing the real work on a cache miss.
    inner: Box<dyn TextShaper>,
    /// Memoized shaped runs, keyed by every input that affects the output.
    cache: RefCell<HashMap<ShapeKey, GlyphRun>>,
    /// The maximum number of cached runs before a flush.
    capacity: usize,
}

impl CachingShaper {
    /// The default cache capacity: the number of distinct shaped runs kept
    /// before the cache is flushed.
    pub const MAX_ENTRIES: usize = 1024;

    /// Wraps `inner` in a cache with the default capacity
    /// ([`MAX_ENTRIES`](Self::MAX_ENTRIES)).
    #[must_use]
    pub fn new(inner: Box<dyn TextShaper>) -> Self {
        Self::with_capacity(inner, Self::MAX_ENTRIES)
    }

    /// Wraps `inner` in a cache holding at most `capacity` runs (a `capacity` of
    /// zero is treated as one, so a single entry can always be cached).
    #[must_use]
    pub fn with_capacity(inner: Box<dyn TextShaper>, capacity: usize) -> Self {
        Self {
            inner,
            cache: RefCell::new(HashMap::new()),
            capacity: capacity.max(1),
        }
    }

    /// Returns the number of currently cached runs (for tests and diagnostics).
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.borrow().len()
    }

    /// Returns whether the cache currently holds no runs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.borrow().is_empty()
    }
}

impl TextShaper for CachingShaper {
    fn shape(&self, request: &ShapingRequest) -> GlyphRun {
        let key = ShapeKey::from_request(request);
        if let Some(run) = self.cache.borrow().get(&key) {
            return run.clone();
        }
        // Miss: shape once through the inner engine, then memoize.
        let run = self.inner.shape(request);
        let mut cache = self.cache.borrow_mut();
        if cache.len() >= self.capacity && !cache.contains_key(&key) {
            cache.clear();
        }
        cache.insert(key, run.clone());
        run
    }

    fn can_shape(&self, face: &crate::font::FontFace) -> bool {
        self.inner.can_shape(face)
    }

    fn line_metrics(&self, font_size: f32) -> LineMetrics {
        self.inner.line_metrics(font_size)
    }

    fn line_metrics_for(&self, font: crate::font::FontId, font_size: f32) -> LineMetrics {
        self.inner.line_metrics_for(font, font_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontFace, FontMetrics, FontSource, GenericFamily};
    use crate::shaping::MockTextShaper;
    use std::cell::Cell;

    fn mono_face() -> FontFace {
        FontFace::new(
            FontFamily::generic(GenericFamily::Monospace),
            FontAttributes::default(),
            FontSource::system("mono"),
            FontMetrics::new(1000, 800.0, -200.0, 0.0),
        )
    }

    // A counting shaper wrapped in a shared cell so the test can read its call
    // count after moving it into the CachingShaper as a Box<dyn TextShaper>.
    #[derive(Clone)]
    struct SharedCounter(std::rc::Rc<Cell<usize>>);

    struct SharedCountingShaper(SharedCounter);

    impl TextShaper for SharedCountingShaper {
        fn shape(&self, request: &ShapingRequest) -> GlyphRun {
            self.0.0.set(self.0.0.get() + 1);
            MockTextShaper::new().shape(request)
        }
        fn can_shape(&self, _face: &FontFace) -> bool {
            true
        }
    }

    #[test]
    fn identical_requests_shape_once() {
        let counter = SharedCounter(std::rc::Rc::new(Cell::new(0)));
        let shaper = CachingShaper::new(Box::new(SharedCountingShaper(counter.clone())));
        let request = ShapingRequest::new("hello", mono_face(), 16.0);
        let a = shaper.shape(&request);
        let b = shaper.shape(&request);
        assert_eq!(counter.0.get(), 1, "second identical request must be a hit");
        assert_eq!(a, b);
    }

    #[test]
    fn differing_requests_shape_twice_no_collision() {
        let counter = SharedCounter(std::rc::Rc::new(Cell::new(0)));
        let shaper = CachingShaper::new(Box::new(SharedCountingShaper(counter.clone())));
        // Vary each keyed dimension independently; each must miss.
        let _ = shaper.shape(&ShapingRequest::new("a", mono_face(), 16.0));
        let _ = shaper.shape(&ShapingRequest::new("b", mono_face(), 16.0)); // text
        let _ = shaper.shape(&ShapingRequest::new("a", mono_face(), 18.0)); // size
        let _ = shaper.shape(
            &ShapingRequest::new("a", mono_face(), 16.0).with_direction(TextDirection::RightToLeft), // direction
        );
        let _ = shaper.shape(&ShapingRequest::new("a", mono_face(), 16.0).with_language("en")); // language
        assert_eq!(
            counter.0.get(),
            5,
            "each distinct key must reach the inner shaper"
        );
    }

    #[test]
    fn cached_run_equals_direct_output() {
        let inner = MockTextShaper::new();
        let cached = CachingShaper::new(Box::new(MockTextShaper::new()));
        for text in ["", "x", "aé漢", "hello world"] {
            let request = ShapingRequest::new(text, mono_face(), 14.0);
            let direct = inner.shape(&request);
            let via_cache = cached.shape(&request);
            let via_cache_again = cached.shape(&request);
            assert_eq!(direct, via_cache);
            assert_eq!(direct, via_cache_again);
        }
    }

    #[test]
    fn capacity_bound_holds_and_stays_correct() {
        let cap = 4;
        let cached = CachingShaper::with_capacity(Box::new(MockTextShaper::new()), cap);
        // Insert well past the cap; the map must never exceed it.
        for i in 0..50 {
            let text = format!("t{i}");
            let request = ShapingRequest::new(text.as_str(), mono_face(), 12.0);
            let run = cached.shape(&request);
            // Still correct despite the bound.
            assert_eq!(run, MockTextShaper::new().shape(&request));
            assert!(
                cached.len() <= cap,
                "cache exceeded capacity: {}",
                cached.len()
            );
        }
        assert!(cached.len() <= cap);
    }

    #[test]
    fn line_metrics_and_can_shape_delegate() {
        let cached = CachingShaper::new(Box::new(MockTextShaper::new()));
        assert!(cached.can_shape(&mono_face()));
        assert_eq!(
            cached.line_metrics(16.0),
            MockTextShaper::new().line_metrics(16.0)
        );
    }
}
