use async_lock::{Mutex, RwLock};
use async_trait::async_trait;
use core::marker::PhantomData;
use locking::{
    async_lock::{AsyncLock, AsyncSendLock, WeakAsyncSendLock},
    lock::WeakLock,
    AsyncLockApi, LockApiReadGuard, LockApiReadWriteGuard,
};
use std::future::Future;

use crate::{Downgrade, State, StateError, StateTrait};

#[async_trait]
pub trait AsyncIntoInner<T> {
    async fn into_inner(self) -> Option<T>;
    async fn replace_inner(&self, other: T) -> Option<T>;
}

#[async_trait]
pub trait AsyncStateTrait<T> {
    async fn read<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&T) -> U + Send,
        U: Future + Send;

    async fn write<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnMut(&mut T) -> U + Send,
        U: Future + Send;

    async fn is_valid(&self) -> bool;
}

#[async_trait(?Send)]
pub trait LocalAsyncStateTrait<T> {
    async fn read<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&T) -> U,
        U: Future;

    async fn write<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&mut T) -> U,
        U: Future;

    async fn is_valid(&self) -> bool;
}

#[async_trait(?Send)]
impl<T> LocalAsyncStateTrait<T> for State<T> {
    async fn read<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&T) -> U,
        U: Future,
    {
        match <Self as StateTrait<T>>::read(self, func) {
            Ok(ret) => Ok(ret.await),
            Err(err) => Err(err),
        }
    }

    async fn write<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&mut T) -> U,
        U: Future,
    {
        match <Self as StateTrait<T>>::write(self, func) {
            Ok(ret) => Ok(ret.await),
            Err(err) => Err(err),
        }
    }

    async fn is_valid(&self) -> bool {
        <Self as StateTrait<T>>::is_valid(self)
    }
}

pub type AsyncMutexState<T> = AsyncLockState<T, AsyncSendLock<Mutex<Option<T>>>>;

pub type WeakAsyncMutexState<T> = WeakAsyncLockState<T, WeakAsyncSendLock<Mutex<Option<T>>>>;

pub type AsyncRwLockState<T> = AsyncLockState<T, RwLock<Option<T>>>;

pub struct AsyncLockState<T, L> {
    lock: L,
    _t: PhantomData<T>,
}

impl<T, L> AsyncLockState<T, L>
where
    for<'a> L: AsyncLock<'a, Option<T>>,
{
    pub fn new(state: T) -> AsyncLockState<T, L> {
        AsyncLockState {
            lock: L::new(Some(state)),
            _t: PhantomData,
        }
    }
}

#[async_trait]
impl<T, L> AsyncStateTrait<T> for AsyncLockState<T, L>
where
    L: Sync + Send,
    for<'a> L: AsyncLockApi<'a, Option<T>>,
    T: Send + Sync,
{
    async fn read<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&T) -> U + Send,
        U: Future + Send,
    {
        let future = self.lock.read();
        let m = future.await;
        let ret = match m.get() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret).await)
    }

    async fn write<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&mut T) -> U + Send,
        U: Future + Send,
    {
        let future = self.lock.write();
        let mut m = future.await;
        let ret = match m.get_mut() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret).await)
    }

    async fn is_valid(&self) -> bool {
        let future = self.lock.read();
        future.await.get().is_some()
    }
}

#[async_trait]
impl<T, L> AsyncIntoInner<T> for AsyncLockState<T, L>
where
    for<'a> L: AsyncLockApi<'a, Option<T>> + Sync + Send,
    T: Send + Sync,
{
    async fn into_inner(self) -> Option<T> {
        let future = self.lock.write();
        future.await.get_mut().take()
    }
    async fn replace_inner(&self, other: T) -> Option<T> {
        let future = self.lock.write();
        future.await.get_mut().replace(other)
    }
}

impl<T, L> Downgrade for AsyncLockState<T, L>
where
    L: locking::lock::Downgrade,
    L::Weak: Send + Sync,
    for<'a> L: AsyncLock<'a, Option<T>>,
    T: Send + Sync,
{
    type Output = WeakAsyncLockState<T, L::Weak>;

    fn downgrade(&self) -> Self::Output {
        WeakAsyncLockState {
            lock: self.lock.downgrade(),
            _t: PhantomData,
        }
    }
}

//

pub struct WeakAsyncLockState<T, L>
where
    L: WeakLock,
{
    lock: L,
    _t: PhantomData<T>,
}

#[async_trait]
impl<T, L> AsyncStateTrait<T> for WeakAsyncLockState<T, L>
where
    L: Sync + Send,
    L: WeakLock,
    for<'a> L::Output: AsyncLockApi<'a, Option<T>> + Send,
    T: Send + Sync,
{
    async fn read<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&T) -> U + Send,
        U: Future + Send,
    {
        let inner = match self.lock.upgrade() {
            Some(i) => i,
            None => return Err(StateError::Upgrade),
        };

        let future = inner.read();
        let m = future.await;
        let ret = match m.get() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret).await)
    }

    async fn write<F, U>(&self, func: F) -> Result<U::Output, StateError>
    where
        F: FnOnce(&mut T) -> U + Send,
        U: Future + Send,
    {
        let inner = match self.lock.upgrade() {
            Some(i) => i,
            None => return Err(StateError::Upgrade),
        };

        let future = inner.write();
        let mut m = future.await;
        let ret = match m.get_mut() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret).await)
    }

    async fn is_valid(&self) -> bool {
        let inner = match self.lock.upgrade() {
            Some(i) => i,
            None => return false,
        };
        let future = inner.read();
        let m = future.await;
        m.get().is_some()
    }
}

#[async_trait]
impl<T, L> AsyncIntoInner<T> for WeakAsyncLockState<T, L>
// where
//     for<'a> L: AsyncLockApi<'a, Option<T>> + Sync + Send,
//     T: Send + Sync,
where
    T: Send + Sync,
    L: WeakLock + Send + Sync,
    for<'a> L::Output: AsyncLockApi<'a, Option<T>> + Send + Sync,
{
    async fn into_inner(self) -> Option<T> {
        let inner = match self.lock.upgrade() {
            Some(i) => i,
            None => return None,
        };
        let future = inner.write();
        let mut m = future.await;
        m.get_mut().take()
    }

    async fn replace_inner(&self, other: T) -> Option<T> {
        let inner = match self.lock.upgrade() {
            Some(i) => i,
            None => return Some(other),
        };
        let future = inner.write();
        let mut m = future.await;
        m.get_mut().replace(other)
    }
}
