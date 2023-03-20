use crate::transport::{
    func::{CallbackMut, CallbackMutExt},
    reply::Reply,
    IntoListener, Listener, Subscription, Transport,
};
use generational_arena::{Arena, Index};
use state::{Downgrade, State, StateTrait, WeakState};

pub struct DirectListener<'a, E> {
    func: Box<dyn CallbackMut<E, Output = bool> + 'a>,
}

impl<'a, E> DirectListener<'a, E> {
    pub fn new<F>(func: F) -> DirectListener<'a, E>
    where
        E: 'a,
        F: CallbackMut<E> + 'a,
        F::Output: Reply,
    {
        DirectListener {
            func: Box::new(func.map(|ret: F::Output| ret.carry_on())),
        }
    }
}

impl<'a, E> CallbackMut<E> for DirectListener<'a, E> {
    type Output = bool;

    fn call_mut(&mut self, input: &E) -> Self::Output {
        self.func.call_mut(input)
    }
}

impl<'a, E> Listener<'a, E> for DirectListener<'a, E> {}

pub struct DirectSubscription<'a, E> {
    arena: WeakState<Arena<DirectListener<'a, E>>>,
    idx: Option<Index>,
}

impl<'a, E> Subscription for DirectSubscription<'a, E> {
    fn close(self) {
        drop(self)
    }

    fn detach(mut self) {
        self.idx = None;
    }
}

impl<'a, E> Drop for DirectSubscription<'a, E> {
    fn drop(&mut self) {
        let idx = match self.idx.take() {
            Some(idx) => idx,
            None => return,
        };

        self.arena.write(|arena| arena.remove(idx)).ok();
    }
}

pub struct Direct<'a, E> {
    listener: State<Arena<DirectListener<'a, E>>>,
}

impl<'a, E> Clone for Direct<'a, E> {
    fn clone(&self) -> Self {
        Direct {
            listener: self.listener.clone(),
        }
    }
}

impl<'a, E> Transport<'a, E> for Direct<'a, E> {
    type Listener = DirectListener<'a, E>;

    type Subscription = DirectSubscription<'a, E>;

    fn create_listener<L>(&self, listener: L) -> Self::Subscription
    where
        L: IntoListener<'a, Self::Listener, E>,
    {
        let listener = listener.into_listener();
        let idx = self.listener.write(|arena| arena.insert(listener)).unwrap();

        DirectSubscription {
            idx: Some(idx),
            arena: self.listener.downgrade(),
        }
    }

    fn trigger(&self, event: E) {
        self.listener
            .write(|arena| {
                for (_idx, next) in arena.iter_mut() {
                    next.func.call_mut(&event);
                }
            })
            .ok();
    }
}

impl<'a, E> Default for Direct<'a, E> {
    fn default() -> Self {
        Direct {
            listener: State::new(Arena::default()),
        }
    }
}
