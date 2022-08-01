use state::{MutexState, StateTrait};
use threadpool::ThreadPool;

use crate::{
    direct::sync::{SyncDirectSubscription, SyncListener},
    SyncDirect, Transport,
};

pub struct Pool<E: 'static> {
    thpool: SyncThreadPool,
    trigger: SyncDirect<E>,
}

impl<E> Clone for Pool<E> {
    fn clone(&self) -> Self {
        Pool {
            thpool: self.thpool.clone(),
            trigger: self.trigger.clone(),
        }
    }
}

impl<E> Default for Pool<E> {
    fn default() -> Self {
        Pool::new(num_cpus::get())
    }
}

impl<E> Pool<E> {
    pub fn new(num: usize) -> Pool<E> {
        Pool::new_with(SyncThreadPool::new(num))
    }

    pub fn new_with(thpool: impl Into<SyncThreadPool>) -> Pool<E> {
        Pool {
            thpool: thpool.into(),
            trigger: SyncDirect::default(),
        }
    }
}

impl<E> Transport<'static, E> for Pool<E>
where
    E: Send + 'static,
{
    type Listener = SyncListener<E>;

    type Subscription = SyncDirectSubscription<E>;

    fn create_listener<L>(&self, listener: L) -> Self::Subscription
    where
        L: crate::IntoListener<'static, Self::Listener, E>,
    {
        self.trigger.create_listener(listener)
    }

    fn trigger(&self, event: E) {
        let trigger = self.trigger.clone();
        self.thpool.execute(move || {
            trigger.trigger(event);
        });
    }
}

/// A Send + Sync thread pool.
#[derive(Clone)]
pub struct SyncThreadPool {
    pool: MutexState<ThreadPool>,
}

impl SyncThreadPool {
    /// Create a new thread pool with the specified size.
    pub fn new(num_threads: usize) -> Self {
        Self {
            pool: MutexState::new(ThreadPool::new(num_threads)),
        }
    }

    /// Execute a job on the thread pool.
    pub fn execute(&self, job: impl FnOnce() + Send + 'static) {
        self.pool.read(move |thread| thread.execute(job)).ok();
    }
}

impl From<ThreadPool> for SyncThreadPool {
    fn from(th: ThreadPool) -> Self {
        SyncThreadPool {
            pool: MutexState::new(th),
        }
    }
}
