#[cfg(not(feature = "std"))]
use alloc::rc::{Rc, Weak};
use core::cell::RefCell;
#[cfg(feature = "std")]
use std::rc::{Rc, Weak};

use crate::{Downgrade, IntoInner, StateError};

pub trait StateTrait<T> {
    fn read<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&T) -> U;

    fn write<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&mut T) -> U;

    fn is_valid(&self) -> bool;
}

#[derive(Debug, PartialEq)]
pub struct State<T> {
    inner: Rc<RefCell<Option<T>>>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        State {
            inner: self.inner.clone(),
        }
    }
}

impl<T> State<T> {
    pub fn new(state: T) -> State<T> {
        State {
            inner: Rc::new(RefCell::new(Some(state))),
        }
    }
}

impl<T> StateTrait<T> for State<T> {
    fn read<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&T) -> U,
    {
        let m = self.inner.as_ref().borrow();
        let ret = match &*m {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret))
    }

    fn write<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&mut T) -> U,
    {
        let mut m = self.inner.as_ref().borrow_mut();
        let ret = match &mut *m {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret))
    }

    fn is_valid(&self) -> bool {
        self.inner.borrow().is_some()
    }
}

impl<T> IntoInner<T> for State<T> {
    fn into_inner(self) -> Option<T> {
        self.inner.borrow_mut().take()
    }

    fn replace_inner(&self, other: T) -> Option<T> {
        self.inner.borrow_mut().replace(other)
    }
}

impl<T> Downgrade for State<T> {
    type Output = WeakState<T>;

    fn downgrade(&self) -> Self::Output {
        WeakState {
            inner: Rc::downgrade(&self.inner),
        }
    }
}

//

#[derive(Debug)]
pub struct WeakState<T> {
    inner: Weak<RefCell<Option<T>>>,
}

impl<T> Clone for WeakState<T> {
    fn clone(&self) -> Self {
        WeakState {
            inner: self.inner.clone(),
        }
    }
}

impl<T> StateTrait<T> for WeakState<T> {
    fn read<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&T) -> U,
    {
        let inner = match self.inner.upgrade() {
            Some(i) => i,
            None => return Err(StateError::Upgrade),
        };

        let m = inner.borrow();
        let ret = match &*m {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret))
    }

    fn write<F, U>(&self, func: F) -> Result<U, StateError>
    where
        F: FnOnce(&mut T) -> U,
    {
        let inner = match self.inner.upgrade() {
            Some(i) => i,
            None => return Err(StateError::Upgrade),
        };

        let mut m = inner.borrow_mut();

        let ret = match &mut *m {
            Some(ret) => ret,
            None => return Err(StateError::Empty),
        };

        Ok(func(ret))
    }

    fn is_valid(&self) -> bool {
        match self.inner.upgrade() {
            Some(ret) => ret.borrow().is_some(),

            _ => false,
        }
    }
}

impl<T> IntoInner<T> for WeakState<T> {
    fn into_inner(self) -> Option<T> {
        match self.inner.upgrade() {
            Some(ret) => ret.borrow_mut().take(),
            _ => None,
        }
    }

    fn replace_inner(&self, other: T) -> Option<T> {
        match self.inner.upgrade() {
            Some(ret) => ret.borrow_mut().replace(other),
            _ => None,
        }
    }
}
