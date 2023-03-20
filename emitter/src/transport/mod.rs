mod func;
mod into_listener;
mod reply;

mod r#async;

pub mod direct;
pub use direct::*;

#[cfg(feature = "thread")]
pub mod thread;
#[cfg(feature = "thread")]
pub use thread::Thread;

#[cfg(feature = "threadpool")]
pub mod pool;
#[cfg(feature = "threadpool")]
pub use pool::Pool;

pub trait Listener<'a, E> {}

pub trait Subscription {
    fn close(self);
    fn detach(self);
}

pub trait IntoListener<'a, L, E>
where
    L: Listener<'a, E>,
{
    fn into_listener(self) -> L;
}

pub trait Transport<'a, E> {
    type Listener: Listener<'a, E>;
    type Subscription: Subscription;

    fn create_listener<L>(&self, listener: L) -> Self::Subscription
    where
        L: IntoListener<'a, Self::Listener, E>;

    fn trigger(&self, event: E);
}
