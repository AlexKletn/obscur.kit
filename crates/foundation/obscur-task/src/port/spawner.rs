#![doc = include_str!("spawner.doc/_module.md")]

use std::future::Future;
use std::pin::Pin;

#[doc = include_str!("spawner.doc/BoxFuture.md")]
pub type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[doc = include_str!("spawner.doc/Spawner.md")]
pub trait Spawner {
    #[doc = include_str!("spawner.doc/spawn.md")]
    fn spawn(&self, future: BoxFuture);
}
