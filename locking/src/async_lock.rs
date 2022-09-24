// use crate::Downgrade;

use super::async_locking::AsyncLockApi;
use super::lock::{Downgrade, WeakLock};
use async_trait::async_trait;

#[cfg(not(feature = "std"))]
use alloc::sync::{Arc, Weak as ArcWeak};
#[cfg(feature = "std")]
use std::sync::{Arc, Weak as ArcWeak};

pub trait AsyncLock<'a, T: 'a>: AsyncLockApi<'a, T> + Downgrade + Clone {}

#[derive(Debug)]
pub struct AsyncSendLock<L> {
    lock: Arc<L>,
}

unsafe impl<L: Send> Send for AsyncSendLock<L> {}

unsafe impl<L: Sync> Sync for AsyncSendLock<L> {}

impl<L> Clone for AsyncSendLock<L> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock.clone(),
            // _t: self._t.clone(),
        }
    }
}

#[async_trait]
impl<'a, T, L> AsyncLockApi<'a, T> for AsyncSendLock<L>
where
    L: AsyncLockApi<'a, T> + Send + Sync,
{
    type ReadGuard = <L as AsyncLockApi<'a, T>>::ReadGuard;

    type ReadWriteGuard = <L as AsyncLockApi<'a, T>>::ReadWriteGuard;

    async fn read(&'a self) -> Self::ReadGuard {
        self.lock.read().await
    }

    async fn write(&'a self) -> Self::ReadWriteGuard {
        self.lock.write().await
    }

    fn new(inner: T) -> Self {
        AsyncSendLock {
            lock: Arc::new(L::new(inner)),
        }
    }
}

impl<'a, L, T: 'a> AsyncLock<'a, T> for AsyncSendLock<L> where L: AsyncLockApi<'a, T> + Send + Sync {}

impl<L> Downgrade for AsyncSendLock<L> {
    type Weak = WeakAsyncSendLock<L>;

    fn downgrade(&self) -> Self::Weak {
        WeakAsyncSendLock(Arc::downgrade(&self.lock))
    }
}

#[derive(Debug)]
pub struct WeakAsyncSendLock<L>(ArcWeak<L>);

unsafe impl<L: Send> Send for WeakAsyncSendLock<L> {}

unsafe impl<L: Sync> Sync for WeakAsyncSendLock<L> {}

impl<L> Clone for WeakAsyncSendLock<L> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<L> WeakLock for WeakAsyncSendLock<L> {
    type Output = AsyncSendLock<L>;
    fn upgrade(&self) -> Option<AsyncSendLock<L>> {
        match self.0.upgrade() {
            Some(lock) => Some(AsyncSendLock { lock }),
            None => None,
        }
    }
}
