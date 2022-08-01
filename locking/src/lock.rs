#[cfg(not(feature = "std"))]
use alloc::{
    rc::{Rc, Weak as RcWeak},
    sync::{Arc, Weak as ArcWeak},
};
#[cfg(feature = "std")]
use std::{
    rc::{Rc, Weak as RcWeak},
    sync::{Arc, Weak as ArcWeak},
};

use crate::LockApi;

pub trait Downgrade {
    type Weak: WeakLock<Output = Self>;
    fn downgrade(&self) -> Self::Weak;
}

pub trait Lock<'a, T: 'a>: LockApi<'a, T> + Downgrade + Clone {}

pub trait WeakLock: Clone {
    type Output;

    fn upgrade(&self) -> Option<Self::Output>;
}

pub struct SendLock<L> {
    lock: Arc<L>,
}

impl<'a, L, T> LockApi<'a, T> for SendLock<L>
where
    L: LockApi<'a, T>,
{
    type ReadGuard = L::ReadGuard;

    type ReadWriteGuard = L::ReadWriteGuard;

    fn read(&'a self) -> Self::ReadGuard {
        self.lock.read()
    }

    fn write(&'a self) -> Self::ReadWriteGuard {
        self.lock.write()
    }

    fn new(inner: T) -> Self {
        SendLock {
            lock: Arc::new(L::new(inner)),
        }
    }
}

impl<L> Clone for SendLock<L> {
    fn clone(&self) -> Self {
        SendLock {
            lock: self.lock.clone(),
        }
    }
}

impl<'a, L, T: 'a> Lock<'a, T> for SendLock<L> where L: LockApi<'a, T> {}

impl<L> Downgrade for SendLock<L> {
    type Weak = WeakSendLock<L>;

    fn downgrade(&self) -> Self::Weak {
        WeakSendLock(Arc::downgrade(&self.lock))
    }
}

pub struct WeakSendLock<L>(ArcWeak<L>);

impl<L> WeakLock for WeakSendLock<L> {
    type Output = SendLock<L>;

    fn upgrade(&self) -> Option<Self::Output> {
        self.0.upgrade().map(|lock| SendLock { lock })
    }
}

impl<L> Clone for WeakSendLock<L> {
    fn clone(&self) -> Self {
        WeakSendLock(self.0.clone())
    }
}

//

pub struct NonSendLock<L> {
    lock: Rc<L>,
}

impl<'a, L, T> LockApi<'a, T> for NonSendLock<L>
where
    L: LockApi<'a, T>,
{
    type ReadGuard = L::ReadGuard;

    type ReadWriteGuard = L::ReadWriteGuard;

    fn read(&'a self) -> Self::ReadGuard {
        self.lock.read()
    }

    fn write(&'a self) -> Self::ReadWriteGuard {
        self.lock.write()
    }

    fn new(inner: T) -> Self {
        NonSendLock {
            lock: Rc::new(L::new(inner)),
        }
    }
}

impl<L> Clone for NonSendLock<L> {
    fn clone(&self) -> Self {
        NonSendLock {
            lock: self.lock.clone(),
        }
    }
}

impl<'a, L, T: 'a> Lock<'a, T> for NonSendLock<L> where L: LockApi<'a, T> {}

impl<L> Downgrade for NonSendLock<L> {
    type Weak = WeakNonSendLock<L>;

    fn downgrade(&self) -> Self::Weak {
        WeakNonSendLock(Rc::downgrade(&self.lock))
    }
}

pub struct WeakNonSendLock<L>(RcWeak<L>);

impl<L> WeakLock for WeakNonSendLock<L> {
    type Output = NonSendLock<L>;

    fn upgrade(&self) -> Option<Self::Output> {
        self.0.upgrade().map(|lock| NonSendLock { lock })
    }
}

impl<L> Clone for WeakNonSendLock<L> {
    fn clone(&self) -> Self {
        WeakNonSendLock(self.0.clone())
    }
}
