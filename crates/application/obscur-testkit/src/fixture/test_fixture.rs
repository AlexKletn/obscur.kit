#![doc = include_str!("test_fixture.doc/_module.md")]

use obscur_geometry::Size;
use obscur_identity::RuntimeId;
use obscur_runtime::UiRuntime;

#[doc = include_str!("test_fixture.doc/TestFixture.md")]
#[derive(Debug)]
pub struct TestFixture;

impl TestFixture {
    #[doc = include_str!("test_fixture.doc/RUNTIME_ID.md")]
    pub const RUNTIME_ID: RuntimeId = RuntimeId::from_raw(1);

    #[doc = include_str!("test_fixture.doc/VIEWPORT.md")]
    pub const VIEWPORT: Size = Size::new(300.0, 300.0);

    #[doc = include_str!("test_fixture.doc/runtime.md")]
    #[must_use]
    pub fn runtime() -> UiRuntime {
        UiRuntime::new(Self::RUNTIME_ID, Self::VIEWPORT)
    }
}
