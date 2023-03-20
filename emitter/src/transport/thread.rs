use std::thread::JoinHandle;

use crossbeam_channel::{unbounded, Sender};
use state::{Downgrade, IntoInner, MutexState, WeakMutexState};

use crate::{
    direct::sync::{SyncDirectSubscription, SyncListener},
    Subscription, SyncDirect, Transport,
};

enum Message<E: 'static> {
    Event(E),
    Listener(SyncListener<E>, WeakMutexState<SyncDirectSubscription<E>>),
}

pub struct Thread<E: 'static> {
    #[allow(unused)]
    handle: JoinHandle<()>,
    channel: Sender<Message<E>>,
}

impl<E> Default for Thread<E>
where
    E: 'static + Send,
{
    fn default() -> Self {
        Thread::new()
    }
}

impl<E> Thread<E>
where
    E: 'static + Send,
{
    pub fn new() -> Thread<E> {
        let (channel, rx) = unbounded();

        let handle = std::thread::spawn(move || {
            let trigger = SyncDirect::<E>::default();

            while let Ok(next) = rx.recv() {
                match next {
                    Message::Event(event) => {
                        trigger.trigger(event);
                    }
                    Message::Listener(listener, subs) => {
                        let subscription = trigger.create_listener(listener);
                        subs.replace_inner(subscription);
                    }
                }
            }
        });

        Thread { handle, channel }
    }
}

pub struct ThreadSubscription<E: 'static>(MutexState<SyncDirectSubscription<E>>);

impl<E> Subscription for ThreadSubscription<E> {
    fn close(self) {
        if let Some(inner) = self.0.into_inner() {
            inner.close()
        }
    }

    fn detach(self) {
        if let Some(inner) = self.0.into_inner() {
            inner.detach();
        }
    }
}

impl<E> Transport<'static, E> for Thread<E> {
    type Listener = SyncListener<E>;

    type Subscription = ThreadSubscription<E>;

    fn create_listener<L>(&self, listener: L) -> Self::Subscription
    where
        L: crate::IntoListener<'static, Self::Listener, E>,
    {
        let subscription = ThreadSubscription(MutexState::default());

        self.channel
            .send(Message::Listener(
                listener.into_listener(),
                subscription.0.downgrade(),
            ))
            .ok();

        subscription
    }

    fn trigger(&self, event: E) {
        self.channel.send(Message::Event(event)).ok();
    }
}
