use crate::DirectListener;
#[allow(unused_imports)]
use crate::{IntoListener, Listener};

#[cfg(feature = "sync")]
use crate::direct::sync::SyncListener;

#[allow(unused_imports)]
use super::{
    func::{Callback, CallbackExt, CallbackMut},
    reply::Reply,
};

impl<'a, E, F> IntoListener<'a, DirectListener<'a, E>, E> for F
where
    E: 'a,
    F: CallbackMut<E> + 'a,
    F::Output: Reply,
{
    fn into_listener(self) -> DirectListener<'a, E> {
        DirectListener::new(self)
    }
}

#[cfg(feature = "sync")]
impl<E, F> IntoListener<'static, SyncListener<E>, E> for F
where
    E: 'static,
    F: Callback<E> + Send + Sync + 'static,
    F::Output: Reply,
{
    fn into_listener(self) -> SyncListener<E> {
        SyncListener::new(self.map(|m: F::Output| m.carry_on()))
    }
}
