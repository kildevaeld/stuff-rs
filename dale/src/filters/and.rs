use super::generic::{Combine, Extract, HList, Tuple};
use crate::{IntoOutcome, Outcome, Service};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use either::Either;
use futures_core::ready;
use pin_project_lite::pin_project;

#[derive(Clone, Copy, Debug)]
pub struct And<T, U> {
    pub(super) first: T,
    pub(super) second: U,
}

impl<T, U> And<T, U> {
    pub fn new(first: T, second: U) -> And<T, U> {
        And { first, second }
    }
}

impl<'a, T, U, R> Service<R> for And<T, U>
where
    // R: Send + Sync,
    T: Service<R>,
    <T::Output as IntoOutcome<R>>::Success: Extract<R>, //+ Send,
    U: Service<R> + Clone,                              //+ Send,
    <U::Output as IntoOutcome<R>>::Success: Extract<R>,
    <<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList:
        Combine<<<<U::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList>, // + Send,
                                                                                                    // <<<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList as Combine<
                                                                                                    //     <<<U::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList,
                                                                                                    // >>::Output as HList>::Tuple: Send,
{
    #[allow(clippy::type_complexity)]
    type Output = Outcome<(
        R,
        <<<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList as Combine<
            <<<U::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList,
        >>::Output as HList>::Tuple,
    ), Either<<T::Output as IntoOutcome<R>>::Failure, <U::Output as IntoOutcome<R>>::Failure>, R>;
    type Future = AndFuture<R, T, U>;

    fn call(&self, req: R) -> Self::Future {
        AndFuture {
            state: State::First {
                future: self.first.call(req),
                next: self.second.clone(),
            },
        }
    }
}

pin_project! {
    pub struct AndFuture<R, T: Service<R>, U: Service<R>>
    where
        <T::Output as IntoOutcome<R>>::Success: Extract<R>,
        <U::Output as IntoOutcome<R>>::Success: Extract<R>,
    {
        #[pin]
        state: State<R, T, U>,
    }
}

pin_project! {
    #[project = StateProj]
    enum State<R, T: Service<R>, U: Service<R>>
    where
        <T::Output as IntoOutcome<R>>::Success: Extract<R>,
        <U::Output as IntoOutcome<R>>::Success: Extract<R>,
    {
        First {
            #[pin]
            future: T::Future,
            next:U
        },
        Second {
            output:Option<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract>,
            #[pin]
            future: U::Future
        },
        Done,
    }
}

impl<R, T, U> Future for AndFuture<R, T, U>
where
    T: Service<R>,
    <T::Output as IntoOutcome<R>>::Success: Extract<R>, // + Send,
    U: Service<R> + Clone,                              // + Send,
    <U::Output as IntoOutcome<R>>::Success: Extract<R>,
    <<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList:
        Combine<<<<U::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList>, // + Send,

                                                                                                    // <<<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList as Combine<
                                                                                                    //     <<<U::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList,
                                                                                                    // >>::Output as HList>::Tuple: Send,
{
    #[allow(clippy::type_complexity)]
    type Output = Outcome<(
        R,
        <<<<<T::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList as Combine<
            <<<U::Output as IntoOutcome<R>>::Success as Extract<R>>::Extract as Tuple>::HList,
        >>::Output as HList>::Tuple,
    ), Either<<T::Output as IntoOutcome<R>>::Failure, <U::Output as IntoOutcome<R>>::Failure>, R>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        loop {
            let pin = self.as_mut().project();
            let (ex1, fut2) = match pin.state.project() {
                StateProj::First {
                    future: first,
                    next: second,
                } => match ready!(first.poll(cx)).into_outcome() {
                    Outcome::Success(ret) => {
                        let (req, first) = ret.unpack();
                        (first, second.call(req))
                    }
                    Outcome::Next(next) => return Poll::Ready(Outcome::Next(next)),
                    Outcome::Failure(err) => {
                        return Poll::Ready(Outcome::Failure(Either::Left(err)))
                    }
                },
                StateProj::Second {
                    output: ex1,
                    future: second,
                } => {
                    let (req, ex2) = match ready!(second.poll(cx)).into_outcome() {
                        Outcome::Success(second) => second.unpack(),
                        Outcome::Failure(err) => {
                            return Poll::Ready(Outcome::Failure(Either::Right(err)))
                        }
                        Outcome::Next(next) => {
                            //
                            return Poll::Ready(Outcome::Next(next));
                        }
                    };

                    let ex3 = ex1.take().unwrap().hlist().combine(ex2.hlist()).flatten();
                    self.set(AndFuture { state: State::Done });
                    return Poll::Ready(Outcome::Success((req, ex3)));
                }
                StateProj::Done => panic!("polled after complete"),
            };

            self.set(AndFuture {
                state: State::Second {
                    output: Some(ex1),
                    future: fut2,
                },
            });
        }
    }
}
