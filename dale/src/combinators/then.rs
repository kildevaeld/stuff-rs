use crate::{types::MapFunc, IntoOutcome, Outcome, Service};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use either::Either;
use futures_core::{ready, Future, TryFuture};
use pin_project_lite::pin_project;

#[derive(Debug, Clone, Copy)]
pub struct Then<S, F> {
    service: S,
    func: F,
}

impl<S, F> Then<S, F> {
    pub fn new(service: S, func: F) -> Then<S, F> {
        Then { service, func }
    }
}

impl<S, F, R> Service<R> for Then<S, F>
where
    S: Service<R>,
    F: MapFunc<
            Result<
                <<S as Service<R>>::Output as IntoOutcome<R>>::Success,
                <<S as Service<R>>::Output as IntoOutcome<R>>::Failure,
            >,
        > + Clone
        + Send,
    F::Output: TryFuture,
{
    type Output = Outcome<
        <F::Output as TryFuture>::Ok,
        Either<<S::Output as IntoOutcome<R>>::Failure, <F::Output as TryFuture>::Error>,
        R,
    >;

    type Future = ThenFuture<S, F, R>;

    fn call(&self, req: R) -> Self::Future {
        let future = self.service.call(req);
        ThenFuture {
            state: ThenFutureState::First {
                future,
                next: self.func.clone(),
            },
        }
    }
}

pin_project! {
    pub struct ThenFuture<S, F, R>
    where
        S: Service<R>,
        F: MapFunc<Result<<<S as Service<R>>::Output as IntoOutcome<R>>::Success,<<S as Service<R>>::Output as IntoOutcome<R>>::Failure>>,
    {
        #[pin]
       state: ThenFutureState<S, F, R>
    }
}

pin_project! {
    #[project = StateProj]
    enum ThenFutureState<S, F, R>
    where
        S: Service<R>,
        F: MapFunc<Result<<<S as Service<R>>::Output as IntoOutcome<R>>::Success,<<S as Service<R>>::Output as IntoOutcome<R>>::Failure>>,
    {
        First {
            #[pin]
            future: S::Future,
            next:F
        },
        Second {
            #[pin]
            future: F::Output
        },
        Done,
    }
}

impl<S, F, R> Future for ThenFuture<S, F, R>
where
    S: Service<R>,
    F: MapFunc<
        Result<
            <<S as Service<R>>::Output as IntoOutcome<R>>::Success,
            <<S as Service<R>>::Output as IntoOutcome<R>>::Failure,
        >,
    >,
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
                            next.call(Ok(ret))
                        }
                        Outcome::Next(next) => return Poll::Ready(Outcome::Next(next)),
                        Outcome::Failure(err) => {
                            //
                            next.call(Err(err))
                        }
                    }
                }
                StateProj::Second { future } => {
                    //
                    let ret = match ready!(future.try_poll(cx)) {
                        Ok(ret) => Outcome::Success(ret),
                        Err(err) => Outcome::Failure(Either::Right(err)),
                    };

                    self.set(ThenFuture {
                        state: ThenFutureState::Done,
                    });

                    return Poll::Ready(ret);
                }
                StateProj::Done => {
                    panic!("polled after complete")
                }
            };

            self.set(ThenFuture {
                state: ThenFutureState::Second { future },
            });
        }
    }
}
