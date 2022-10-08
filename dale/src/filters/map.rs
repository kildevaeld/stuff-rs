use super::generic::{Extract, Func};
use crate::{IntoOutcome, Outcome, Service};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;
use pin_project_lite::pin_project;

#[derive(Clone, Copy, Debug)]
pub struct Map<T, F> {
    pub(super) filter: T,
    pub(super) callback: F,
}

impl<T, F> Map<T, F> {
    pub fn new(filter: T, callback: F) -> Map<T, F> {
        Map { filter, callback }
    }
}

impl<T, F, R> Service<R> for Map<T, F>
where
    T: Service<R>,
    <T::Output as IntoOutcome<R>>::Success: Extract<R>,
    F: Func<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract> + Clone,
{
    type Output = Outcome<(R, (F::Output,)), <T::Output as IntoOutcome<R>>::Failure, R>;
    type Future = MapFuture<T, F, R>;
    #[inline]
    fn call(&self, req: R) -> Self::Future {
        MapFuture {
            extract: self.filter.call(req),
            callback: self.callback.clone(),
        }
    }
}

pin_project! {

    pub struct MapFuture<T: Service<R>, F, R> {
        #[pin]
        extract: T::Future,
        callback: F,
    }

}

impl<T, F, R> Future for MapFuture<T, F, R>
where
    T: Service<R>,
    <T::Output as IntoOutcome<R>>::Success: Extract<R>,
    F: Func<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract> + Clone,
{
    #[allow(clippy::type_complexity)]
    type Output = Outcome<(R, (F::Output,)), <T::Output as IntoOutcome<R>>::Failure, R>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let pin = self.project();
        match ready!(pin.extract.poll(cx)).into_outcome() {
            Outcome::Success(ret) => {
                let (req, ex) = ret.unpack();
                let ex = pin.callback.call(ex);
                Poll::Ready(Outcome::Success((req, (ex,))))
            }
            Outcome::Next(next) => Poll::Ready(Outcome::Next(next)),
            Outcome::Failure(err) => Poll::Ready(Outcome::Failure(err)),
        }
    }
}
