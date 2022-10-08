use crate::{IntoOutcome, Outcome, Service};
use core::{
    future::Future,
    marker::PhantomData,
    task::{self, Poll},
};
use futures_core::ready;
use pin_project_lite::pin_project;

#[derive(Debug)]
pub struct ErrInto<S, E> {
    service: S,
    _e: PhantomData<E>,
}

impl<S: Clone, E> Clone for ErrInto<S, E> {
    fn clone(&self) -> Self {
        ErrInto {
            service: self.service.clone(),
            _e: PhantomData,
        }
    }
}

impl<S: Copy, E> Copy for ErrInto<S, E> {}

unsafe impl<S: Sync, E> Sync for ErrInto<S, E> {}

unsafe impl<S: Send, E> Send for ErrInto<S, E> {}

impl<S, E> ErrInto<S, E> {
    pub fn new<R>(service: S) -> ErrInto<S, E>
    where
        S: Service<R>,
        <S::Output as IntoOutcome<R>>::Failure: Into<E>,
    {
        ErrInto {
            service,
            _e: PhantomData,
        }
    }
}

impl<S, E, R> Service<R> for ErrInto<S, E>
where
    S: Service<R>,
    <S::Output as IntoOutcome<R>>::Failure: Into<E>,
{
    type Output = Outcome<<S::Output as IntoOutcome<R>>::Success, E, R>;

    type Future = ErrIntoService<S::Future, R, E>;

    fn call(&self, req: R) -> Self::Future {
        let fut = self.service.call(req);

        ErrIntoService {
            future: fut,
            _r: PhantomData,
            _e: PhantomData,
        }
    }
}

pin_project! {

    pub struct ErrIntoService<T, R, E> {
        #[pin]
        future: T,
        _r: PhantomData<R>,
        _e: PhantomData<E>
    }
}

unsafe impl<T, R, E> Send for ErrIntoService<T, R, E> where T: Send {}

impl<T, R, E> Future for ErrIntoService<T, R, E>
where
    T: Future,
    T::Output: IntoOutcome<R>,
    <T::Output as IntoOutcome<R>>::Failure: Into<E>,
{
    type Output = Outcome<<T::Output as IntoOutcome<R>>::Success, E, R>;

    fn poll(self: core::pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        let ret = match ready!(this.future.poll(cx)).into_outcome() {
            Outcome::Failure(err) => Outcome::Failure(err.into()),
            Outcome::Next(next) => Outcome::Next(next),

            Outcome::Success(success) => Outcome::Success(success),
        };

        Poll::Ready(ret)
    }
}

#[cfg(test)]
mod tests {

    use core::convert::Infallible;

    pub use super::*;
    pub use crate::ServiceExt;

    use futures_executor::block_on;

    #[derive(PartialEq, Debug)]
    struct Error;

    impl From<Infallible> for Error {
        fn from(_: Infallible) -> Self {
            Error
        }
    }

    #[test]
    fn test_err_into() {
        let service = |req: u32| async move { Outcome::<_, Infallible, _>::Success(req) };

        let ret = block_on(service.err_into::<Error>().call(32u32));

        assert_eq!(ret, Outcome::Success(32));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_boxed() {
        let service = |req: u32| async move { Outcome::<_, Infallible, _>::Success(req) };

        let ret = block_on(service.err_into::<Error>().boxed().call(32u32));

        assert_eq!(ret, Outcome::Success(32));
    }
}
