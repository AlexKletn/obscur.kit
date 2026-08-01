#![doc = include_str!("local_spawner.doc/_module.md")]

use std::future::Future;
use std::pin::Pin;

#[doc = include_str!("local_spawner.doc/LocalBoxFuture.md")]
pub type LocalBoxFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;

#[doc = include_str!("local_spawner.doc/LocalSpawner.md")]
pub trait LocalSpawner {
    #[doc = include_str!("local_spawner.doc/spawn_local.md")]
    fn spawn_local(&self, future: LocalBoxFuture);
}
