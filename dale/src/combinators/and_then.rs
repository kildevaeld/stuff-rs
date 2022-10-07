use crate::{types::MapFunc, IntoOutcome, Outcome, Service};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use either::Either;
use futures_core::{ready, Future, TryFuture};
use pin_project_lite::pin_project;

#[derive(Debug, Clone, Copy)]
pub struct AndThen<S, F> {
    service: S,
    func: F,
}

impl<S, F> AndThen<S, F> {
    pub fn new(service: S, func: F) -> AndThen<S, F> {
        AndThen { service, func }
    }
}

impl<S, F, R> Service<R> for AndThen<S, F>
where
    S: Service<R>,
    F: MapFunc<<<S as Service<R>>::Output as IntoOutcome<R>>::Success> + Clone,
    F::Output: TryFuture,
{
    type Output = Outcome<
        <F::Output as TryFuture>::Ok,
        Either<<S::Output as IntoOutcome<R>>::Failure, <F::Output as TryFuture>::Error>,
        R,
    >;

    type Future = AndThenFuture<S, F, R>;

    fn call(&self, req: R) -> Self::Future {
        let future = self.service.call(req);
        AndThenFuture {
            state: AndThenFutureState::First {
                future,
                next: self.func.clone(),
            },
        }
    }
}

pin_project! {
    pub struct AndThenFuture<S, F, R>
    where
        S: Service<R>,
        F: MapFunc<<<S as Service<R>>::Output as IntoOutcome<R>>::Success>

    {
        #[pin]
       state: AndThenFutureState<S, F, R>
    }
}

pin_project! {
    #[project = StateProj]
    enum AndThenFutureState<S, F, R>
    where
        S: Service<R>,
        F: MapFunc<<<S as Service<R>>::Output as IntoOutcome<R>>::Success>
    {
        First {
            #[pin]
            future: S::Future,
            next:F
        },
        Second {
            // output:Option<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract>,
            #[pin]
            future: F::Output
        },
        Done,
    }
}

impl<S, F, R> Future for AndThenFuture<S, F, R>
where
    S: Service<R>,
    F: MapFunc<<<S as Service<R>>::Output as IntoOutcome<R>>::Success>,
    F::Output: TryFuture,
{
    type Output = Outcome<
        <F::Output as TryFuture>::Ok,
        Either<<S::Output as IntoOutcome<R>>::Failure, <F::Output as TryFuture>::Error>,
        R,
    >;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let pin = self.as_mut().project();

            let future = match pin.state.project() {
                StateProj::First { future, next } => {
                    //
                    match ready!(future.poll(cx)).into_outcome() {
                        Outcome::Success(ret) => {
                            //
                            next.call(ret)
                        }
                        Outcome::Next(next) => return Poll::Ready(Outcome::Next(next)),
                        Outcome::Failure(err) => {
                            return Poll::Ready(Outcome::Failure(Either::Left(err)))
                        }
                    }
                }
                StateProj::Second { future } => {
                    //
                    let ret = match ready!(future.try_poll(cx)) {
                        Ok(ret) => Outcome::Success(ret),
                        Err(err) => Outcome::Failure(Either::Right(err)),
                    };

                    self.set(AndThenFuture {
                        state: AndThenFutureState::Done,
                    });

                    return Poll::Ready(ret);
                }
                StateProj::Done => {
                    panic!("polled after complete")
                }
            };

            self.set(AndThenFuture {
                state: AndThenFutureState::Second { future },
            });
        }
    }
}
