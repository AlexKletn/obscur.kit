//! The identity every fixture runtime shares.

use obscur_geometry::Size;
use obscur_identity::RuntimeId;
use obscur_runtime::UiRuntime;

#[doc = include_str!("../../docs/api/fixture/test_fixture.doc/TestFixture.md")]
#[derive(Debug)]
pub struct TestFixture;

impl TestFixture {
    /// The runtime identity every fixture is built with (spec §3.2 `RuntimeId`).
    pub const RUNTIME_ID: RuntimeId = RuntimeId::from_raw(1);

    #[doc = include_str!("../../docs/api/fixture/test_fixture.doc/VIEWPORT.md")]
    pub const VIEWPORT: Size = Size::new(300.0, 300.0);

    #[doc = include_str!("../../docs/api/fixture/test_fixture.doc/runtime.md")]
    #[must_use]
    pub fn runtime() -> UiRuntime {
        UiRuntime::new(Self::RUNTIME_ID, Self::VIEWPORT)
    }
}
