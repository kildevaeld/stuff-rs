use core::{
    cell::{Ref, RefCell, RefMut},
    ops::{Deref, DerefMut},
};

pub trait LockApiReadGuard<'a, T> {
    fn get(&self) -> &T;
}

pub trait LockApiReadWriteGuard<'a, T>: LockApiReadGuard<'a, T> {
    fn get_mut(&mut self) -> &mut T;
}

pub trait LockApi<'a, T> {
    type ReadGuard: LockApiReadGuard<'a, T>;
    type ReadWriteGuard: LockApiReadWriteGuard<'a, T>;

    fn read(&'a self) -> Self::ReadGuard;

    fn write(&'a self) -> Self::ReadWriteGuard;

    fn new(inner: T) -> Self;
}

impl<'a, T> LockApiReadGuard<'a, T> for Ref<'a, T> {
    fn get(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> LockApiReadGuard<'a, T> for RefMut<'a, T> {
    fn get(&self) -> &T {
        self.deref()
    }
}

impl<'a, T> LockApiReadWriteGuard<'a, T> for RefMut<'a, T> {
    fn get_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'a, T> LockApi<'a, T> for RefCell<T>
where
    T: 'a,
{
    type ReadGuard = Ref<'a, T>;

    type ReadWriteGuard = RefMut<'a, T>;

    fn read(&'a self) -> Self::ReadGuard {
        self.borrow()
    }

    fn write(&'a self) -> Self::ReadWriteGuard {
        self.borrow_mut()
    }

    fn new(inner: T) -> Self {
        RefCell::new(inner)
    }
}

#[cfg(feature = "parking_lot")]
mod parking_lot_impl {
    // Mutex
    use super::*;
    use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

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

    impl<'a, T> LockApi<'a, T> for Mutex<T>
    where
        T: 'a,
    {
        type ReadGuard = MutexGuard<'a, T>;

        type ReadWriteGuard = MutexGuard<'a, T>;

        fn read(&'a self) -> Self::ReadGuard {
            self.lock()
        }

        fn write(&'a self) -> Self::ReadWriteGuard {
            self.lock()
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

    impl<'a, T> LockApi<'a, T> for RwLock<T>
    where
        T: 'a,
    {
        type ReadGuard = RwLockReadGuard<'a, T>;

        type ReadWriteGuard = RwLockWriteGuard<'a, T>;

        fn read(&'a self) -> Self::ReadGuard {
            (*self).read()
        }

        fn write(&'a self) -> Self::ReadWriteGuard {
            (*self).write()
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}

#[cfg(feature = "spin")]
mod spin_impl {
    // Mutex
    use super::*;
    use spin::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

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

    impl<'a, T> LockApi<'a, T> for Mutex<T>
    where
        T: 'a,
    {
        type ReadGuard = MutexGuard<'a, T>;

        type ReadWriteGuard = MutexGuard<'a, T>;

        fn read(&'a self) -> Self::ReadGuard {
            self.lock()
        }

        fn write(&'a self) -> Self::ReadWriteGuard {
            self.lock()
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

    impl<'a, T> LockApi<'a, T> for RwLock<T>
    where
        T: 'a,
    {
        type ReadGuard = RwLockReadGuard<'a, T>;

        type ReadWriteGuard = RwLockWriteGuard<'a, T>;

        fn read(&'a self) -> Self::ReadGuard {
            (*self).read()
        }

        fn write(&'a self) -> Self::ReadWriteGuard {
            (*self).write()
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}

#[cfg(feature = "std")]
mod std_impl {
    // Mutex
    use super::*;
    use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

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

    impl<'a, T> LockApi<'a, T> for Mutex<T>
    where
        T: 'a,
    {
        type ReadGuard = MutexGuard<'a, T>;

        type ReadWriteGuard = MutexGuard<'a, T>;

        fn read(&'a self) -> Self::ReadGuard {
            self.lock().expect("lock")
        }

        fn write(&'a self) -> Self::ReadWriteGuard {
            self.lock().expect("lock")
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

    impl<'a, T> LockApi<'a, T> for RwLock<T>
    where
        T: 'a,
    {
        type ReadGuard = RwLockReadGuard<'a, T>;

        type ReadWriteGuard = RwLockWriteGuard<'a, T>;

        fn read(&'a self) -> Self::ReadGuard {
            (*self).read().expect("read lock")
        }

        fn write(&'a self) -> Self::ReadWriteGuard {
            (*self).write().expect("write lock")
        }

        fn new(inner: T) -> Self {
            RwLock::new(inner)
        }
    }
}
