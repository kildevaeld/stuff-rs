use crate::{Downgrade, IntoInner, StateError, StateTrait};
use core::marker::PhantomData;
use locking::{
    lock::{self, Lock, SendLock, WeakLock, WeakSendLock},
    LockApi, LockApiReadGuard, LockApiReadWriteGuard,
};

#[cfg(feature = "parking_lot")]
use parking_lot::{Mutex, RwLock};

#[cfg(all(not(feature = "parking_lot"), feature = "std"))]
use std::sync::{Mutex, RwLock};

#[cfg(not(feature = "std"))]
use spin::{Mutex, RwLock};

pub type MutexState<T> = LockState<T, SendLock<Mutex<Option<T>>>>;

pub type WeakMutexState<T> = WeakLockState<T, WeakSendLock<Mutex<Option<T>>>>;

pub type RwLockState<T> = LockState<T, SendLock<RwLock<Option<T>>>>;

pub type WeakRwLockState<T> = WeakLockState<T, RwLock<Option<T>>>;

pub struct LockState<T, L>
where
    for<'a> L: Lock<'a, Option<T>>,
{
    inner: L,
    _t: PhantomData<T>,
}

impl<T, L> Clone for LockState<T, L>
where
    for<'a> L: Lock<'a, Option<T>>,
    L: Clone,
{
    fn clone(&self) -> Self {
        LockState {
            inner: self.inner.clone(),
            _t: PhantomData,
        }
    }
}

unsafe impl<T, L: Send> Send for LockState<T, L> where for<'a> L: Lock<'a, Option<T>> {}
unsafe impl<T, L: Sync> Sync for LockState<T, L> where for<'a> L: Lock<'a, Option<T>> {}

impl<T, L> LockState<T, L>
where
    for<'a> L: Lock<'a, Option<T>>,
{
    pub fn new(state: T) -> LockState<T, L> {
        LockState {
            inner: L::new(Some(state)),
            _t: PhantomData,
        }
    }
}

impl<T, L> Default for LockState<T, L>
where
    for<'a> L: Lock<'a, Option<T>>,
{
    fn default() -> Self {
        LockState {
            inner: L::new(None),
            _t: PhantomData,
        }
    }
}

impl<T, L> StateTrait<T> for LockState<T, L>
where
    for<'a> L: Lock<'a, Option<T>>,
{
    fn read<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&T) -> U,
    {
        let m = self.inner.read();
        let ret = match m.get() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(&ret))
    }

    fn write<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&mut T) -> U,
    {
        let mut m = self.inner.write();
        let ret = match m.get_mut() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret))
    }

    fn is_valid(&self) -> bool {
        self.inner.read().get().is_some()
    }
}

impl<T, L> IntoInner<T> for LockState<T, L>
where
    for<'a> L: Lock<'a, Option<T>>,
{
    fn into_inner(self) -> Option<T> {
        self.inner.write().get_mut().take()
    }

    fn replace_inner(&self, other: T) -> Option<T> {
        self.inner.write().get_mut().replace(other)
    }
}

impl<T, L> Downgrade for LockState<T, L>
where
    L: lock::Downgrade,
    for<'a> L: Lock<'a, Option<T>>,
{
    type Output = WeakLockState<T, L::Weak>;

    fn downgrade(&self) -> Self::Output {
        WeakLockState {
            lock: self.inner.downgrade(),
            _t: PhantomData,
        }
    }
}

pub struct WeakLockState<T, L>
where
    L: WeakLock,
{
    lock: L,
    _t: PhantomData<T>,
}

impl<T, L> Clone for WeakLockState<T, L>
where
    L: WeakLock,
{
    fn clone(&self) -> Self {
        WeakLockState {
            lock: self.lock.clone(),
            _t: PhantomData,
        }
    }
}

unsafe impl<T, L: Send> Send for WeakLockState<T, L> where for<'a> L: WeakLock {}
unsafe impl<T, L: Sync> Sync for WeakLockState<T, L> where for<'a> L: WeakLock {}

impl<T, L> StateTrait<T> for WeakLockState<T, L>
where
    L: WeakLock,
    for<'a> L::Output: LockApi<'a, Option<T>>,
{
    fn read<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&T) -> U,
    {
        let inner = match self.lock.upgrade() {
            Some(i) => i,
            None => return Err(StateError::Upgrade),
        };

        let m = inner.read();
        let ret = match m.get() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret))
    }

    fn write<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&mut T) -> U,
    {
        let inner = match self.lock.upgrade() {
            Some(i) => i,
            None => return Err(StateError::Upgrade),
        };

        let mut m = inner.write();

        let ret = match m.get_mut() {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret))
    }

    fn is_valid(&self) -> bool {
        match self.lock.upgrade() {
            Some(ret) => ret.read().get().is_some(),

            _ => false,
        }
    }
}

impl<T, L> IntoInner<T> for WeakLockState<T, L>
where
    L: WeakLock,
    for<'a> L::Output: LockApi<'a, Option<T>>,
{
    fn into_inner(self) -> Option<T> {
        match self.lock.upgrade() {
            Some(ret) => ret.write().get_mut().take(),
            _ => None,
        }
    }

    fn replace_inner(&self, other: T) -> Option<T> {
        match self.lock.upgrade() {
            Some(ret) => ret.write().get_mut().replace(other),
            _ => None,
        }
    }
}
