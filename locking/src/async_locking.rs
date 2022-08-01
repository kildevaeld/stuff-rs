use std::{
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

use async_trait::async_trait;

use crate::locking::{LockApiReadGuard, LockApiReadWriteGuard};

use async_lock::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[async_trait]
pub trait AsyncLockApi<'a, T> {
    type ReadGuard: LockApiReadGuard<'a, T> + Send;
    type ReadWriteGuard: LockApiReadWriteGuard<'a, T> + Send;

    async fn read(&'a self) -> Self::ReadGuard;

    async fn write(&'a self) -> Self::ReadWriteGuard;

    fn new(inner: T) -> Self;
}

#[async_trait(?Send)]
pub trait LocalAsyncLockApi<'a, T> {
    type ReadGuard: LockApiReadGuard<'a, T>;
    type ReadWriteGuard: LockApiReadWriteGuard<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard;

    async fn write(&'a self) -> Self::ReadWriteGuard;

    fn new(inner: T) -> Self;
}

impl<'a, T> LockApiReadGuard<'a, T> for MutexGuard<'a, T> {
    fn get(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> LockApiReadWriteGuard<'a, T> for MutexGuard<'a, T> {
    fn get_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

#[async_trait]
impl<'a, T> AsyncLockApi<'a, T> for Mutex<T>
where
    T: 'a + Send,
{
    type ReadGuard = MutexGuard<'a, T>;

    type ReadWriteGuard = MutexGuard<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard {
        self.lock().await
    }

    async fn write(&'a self) -> Self::ReadWriteGuard {
        self.lock().await
    }

    fn new(inner: T) -> Self {
        Mutex::new(inner)
    }
}

#[async_trait(?Send)]
impl<'a, T> LocalAsyncLockApi<'a, T> for Mutex<T>
where
    T: 'a,
{
    type ReadGuard = MutexGuard<'a, T>;

    type ReadWriteGuard = MutexGuard<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard {
        self.lock().await
    }

    async fn write(&'a self) -> Self::ReadWriteGuard {
        self.lock().await
    }

    fn new(inner: T) -> Self {
        Mutex::new(inner)
    }
}

// RwLock

impl<'a, T> LockApiReadGuard<'a, T> for RwLockReadGuard<'a, T> {
    fn get(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> LockApiReadGuard<'a, T> for RwLockWriteGuard<'a, T> {
    fn get(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> LockApiReadWriteGuard<'a, T> for RwLockWriteGuard<'a, T> {
    fn get_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

#[async_trait]
impl<'a, T> AsyncLockApi<'a, T> for RwLock<T>
where
    T: 'a + Send + Sync,
{
    type ReadGuard = RwLockReadGuard<'a, T>;

    type ReadWriteGuard = RwLockWriteGuard<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard {
        (*self).read().await
    }

    async fn write(&'a self) -> Self::ReadWriteGuard {
        self.write().await
    }

    fn new(inner: T) -> Self {
        RwLock::new(inner)
    }
}

#[async_trait(?Send)]
impl<'a, T> LocalAsyncLockApi<'a, T> for RwLock<T>
where
    T: 'a,
{
    type ReadGuard = RwLockReadGuard<'a, T>;

    type ReadWriteGuard = RwLockWriteGuard<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard {
        (*self).read().await
    }

    async fn write(&'a self) -> Self::ReadWriteGuard {
        self.write().await
    }

    fn new(inner: T) -> Self {
        RwLock::new(inner)
    }
}

#[async_trait(?Send)]
impl<'a, T> LocalAsyncLockApi<'a, T> for RefCell<T>
where
    T: 'a,
{
    type ReadGuard = Ref<'a, T>;

    type ReadWriteGuard = RefMut<'a, T>;

    async fn read(&'a self) -> Self::ReadGuard {
        self.borrow()
    }

    async fn write(&'a self) -> Self::ReadWriteGuard {
        self.borrow_mut()
    }

    fn new(inner: T) -> Self {
        RefCell::new(inner)
    }
}

#[derive(Debug)]
pub struct AsyncLock<T, L> {
    lock: Arc<L>,
    _t: PhantomData<T>,
}

unsafe impl<T, L: Send> Send for AsyncLock<T, L> {}

unsafe impl<T, L: Sync> Sync for AsyncLock<T, L> {}

impl<T, L> Clone for AsyncLock<T, L> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock.clone(),
            _t: self._t.clone(),
        }
    }
}

impl<T, L> AsyncLock<T, L> {
    pub fn downgrade(&self) -> WeakAsyncLock<T, L> {
        WeakAsyncLock {
            lock: Arc::downgrade(&self.lock),
            _t: PhantomData,
        }
    }
}

#[async_trait]
impl<'a, T, L> AsyncLockApi<'a, T> for AsyncLock<T, L>
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
        AsyncLock {
            lock: Arc::new(L::new(inner)),
            _t: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct WeakAsyncLock<T, L> {
    lock: Weak<L>,
    _t: PhantomData<T>,
}

unsafe impl<T, L: Send> Send for WeakAsyncLock<T, L> {}

unsafe impl<T, L: Sync> Sync for WeakAsyncLock<T, L> {}

impl<T, L> Clone for WeakAsyncLock<T, L> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock.clone(),
            _t: self._t.clone(),
        }
    }
}

impl<'a, T, L> WeakAsyncLock<T, L> {
    pub fn upgrade(&self) -> Option<AsyncLock<T, L>> {
        match self.lock.upgrade() {
            Some(lock) => Some(AsyncLock {
                lock,
                _t: PhantomData,
            }),
            None => None,
        }
    }
}
