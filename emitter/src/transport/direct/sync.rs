use crate::{
    transport::{
        func::{Callback, CallbackExt},
        reply::Reply,
    },
    IntoListener, Listener, Subscription, Transport,
};
use generational_arena::{Arena, Index};
use state::{sync, Downgrade, StateTrait};

pub struct SyncListener<E> {
    func: Box<dyn Callback<E, Output = bool> + Send + Sync>,
}

impl<E> SyncListener<E> {
    pub fn new<F>(func: F) -> SyncListener<E>
    where
        E: 'static,
        F: Callback<E> + Send + Sync + 'static,
        F::Output: Reply,
    {
        SyncListener {
            func: Box::new(func.map(|ret: F::Output| ret.carry_on())),
        }
    }
}

impl<E> Listener<'static, E> for SyncListener<E> {}

impl<E> Callback<E> for SyncListener<E> {
    type Output = bool;

    fn call(&self, input: &E) -> Self::Output {
        self.func.call(input)
    }
}

pub struct SyncDirect<E: 'static> {
    listener: sync::MutexState<Arena<SyncListener<E>>>,
}

impl<E> Clone for SyncDirect<E> {
    fn clone(&self) -> Self {
        SyncDirect {
            listener: self.listener.clone(),
        }
    }
}

impl<E> Transport<'static, E> for SyncDirect<E> {
    type Listener = SyncListener<E>;

    type Subscription = SyncDirectSubscription<E>;

    fn create_listener<L>(&self, listener: L) -> Self::Subscription
    where
        L: IntoListener<'static, Self::Listener, E>,
    {
        let listener = listener.into_listener();
        let idx = self
            .listener
            .write(move |listeners| listeners.insert(listener))
            .unwrap();

        SyncDirectSubscription {
            idx: Some(idx),
            arena: self.listener.downgrade(),
        }
    }

    fn trigger(&self, event: E) {
        self.listener
            .read(|arena| {
                for (_idx, next) in arena.iter() {
                    next.func.call(&event);
                }
            })
            .ok();
    }
}

pub struct SyncDirectSubscription<E>
where
    E: 'static,
{
    arena: sync::WeakMutexState<Arena<SyncListener<E>>>,
    idx: Option<Index>,
}

impl<E> Subscription for SyncDirectSubscription<E> {
    fn close(self) {
        drop(self)
    }

    fn detach(mut self) {
        self.idx = None;
    }
}

impl<E> Drop for SyncDirectSubscription<E> {
    fn drop(&mut self) {
        let idx = match self.idx.take() {
            Some(idx) => idx,
            None => return,
        };

        self.arena.write(|arena| arena.remove(idx)).ok();
    }
}

impl<E> Default for SyncDirect<E> {
    fn default() -> Self {
        SyncDirect {
            listener: sync::MutexState::new(Arena::default()),
        }
    }
}
