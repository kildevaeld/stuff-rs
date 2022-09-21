use core::{fmt, marker::PhantomData};

use crate::{Direct, IntoListener, Transport};

pub struct Emitter<E, T> {
    trigger: T,
    _e: PhantomData<E>,
}

impl<E, T> fmt::Debug for Emitter<E, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Emitter")
            .field("trigger", &self.trigger)
            .finish()
    }
}

unsafe impl<E, T: Sync> Sync for Emitter<E, T> {}

unsafe impl<E, T: Send> Send for Emitter<E, T> {}

impl<E, T: Clone> Clone for Emitter<E, T> {
    fn clone(&self) -> Self {
        Self {
            trigger: self.trigger.clone(),
            _e: PhantomData,
        }
    }
}

impl<'a, E> Default for Emitter<E, Direct<'a, E>> {
    fn default() -> Self {
        Emitter::new()
    }
}

impl<'a, E, T> Emitter<E, T>
where
    T: Transport<'a, E>,
{
    pub fn new_with(trigger: T) -> Emitter<E, T> {
        Emitter {
            trigger,
            _e: PhantomData,
        }
    }

    pub fn emit(&self, event: E) {
        self.trigger.trigger(event);
    }

    pub fn listen<L>(&self, listener: L) -> T::Subscription
    where
        L: IntoListener<'a, T::Listener, E>,
    {
        self.trigger.create_listener(listener)
    }
}

impl<'a, E, T> Emitter<E, T>
where
    T: Transport<'a, E>,
    T: Default,
{
    pub fn new() -> Emitter<E, T> {
        Emitter::new_with(T::default())
    }
}
