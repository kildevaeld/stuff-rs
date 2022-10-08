#[cfg(feature = "alloc")]
use crate::boxed::{Box, BoxService, BoxedService, LocalBoxService, LocalBoxedService};
#[cfg(feature = "alloc")]
use crate::outcome::Outcome;
use crate::{
    combinators::{AndThen, ErrInto, MapErr, OrService, Then},
    filters::{And, Combine, Extract, Func, Map, Tuple},
    into_outcome::IntoOutcome,
    middleware::{Middleware, MiddlewareFn, MiddlewareFnService},
    service::Service,
    types::MapFunc,
};
#[cfg(feature = "alloc")]
use core::convert::Infallible;
use core::future::Future;
#[cfg(feature = "alloc")]
use either::Either;
use futures_core::TryFuture;

pub trait ServiceExt<T>: Service<T> {
    fn or<O: Service<T>>(self, service: O) -> OrService<Self, O, T>
    where
        Self: Sized,
    {
        OrService::new(self, service)
    }

    fn then<F>(self, then: F) -> Then<Self, F>
    where
        Self: Sized,
        F: MapFunc<
                Result<
                    <Self::Output as IntoOutcome<T>>::Success,
                    <Self::Output as IntoOutcome<T>>::Failure,
                >,
            > + Clone,
        F::Output: TryFuture,
    {
        Then::new(self, then)
    }

    fn and_then<F>(self, then: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: MapFunc<<Self::Output as IntoOutcome<T>>::Success> + Clone,
        F::Output: TryFuture,
    {
        AndThen::new(self, then)
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn on_err<F, O>(
        self,
        func: F,
    ) -> BoxService<
        'static,
        T,
        Either<<Self::Output as IntoOutcome<T>>::Success, O::Output>,
        Infallible,
    >
    where
        Self: Sized + Clone + Send + Sync + 'static,
        Self::Future: Send,
        <Self as Service<T>>::Output: Send,
        <<Self as Service<T>>::Output as IntoOutcome<T>>::Failure: Send,
        <<Self as Service<T>>::Output as IntoOutcome<T>>::Success: Send,
        T: Send + 'static,
        F: Fn(<Self::Output as IntoOutcome<T>>::Failure) -> O + Clone + Send + Sync + 'static,

        O: Future + Send + 'static,
    {
        (move |req: T| {
            let this = self.clone();
            let func = func.clone();
            async move {
                match this.call(req).await.into_outcome() {
                    Outcome::Failure(err) => Outcome::Success(Either::Right(func(err).await)),
                    Outcome::Next(next) => Outcome::Next(next),
                    Outcome::Success(ret) => Outcome::Success(Either::Left(ret)),
                }
            }
        })
        .boxed()
    }

    // Middlewares

    fn wrap<M>(self, middleware: M) -> M::Service
    where
        M: Middleware<T, Self>,
        Self: Sized,
    {
        middleware.wrap(self)
    }

    fn wrap_fn<F, U>(self, middleware: F) -> MiddlewareFnService<T, F, Self>
    where
        Self: Sized + Clone,
        F: Clone + Fn(Self, T) -> U,
        U: Future,
        U::Output: IntoOutcome<T>,
    {
        self.wrap(MiddlewareFn::new(middleware))
    }

    // Error handling

    fn map_err<F, E>(self, func: F) -> MapErr<F, Self, E>
    where
        Self: Sized,
        F: Fn(<Self::Output as IntoOutcome<T>>::Failure) -> E + Clone,
    {
        MapErr::new(self, func)
    }

    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Failure: Into<E>,
    {
        ErrInto::new(self)
    }

    // Filters

    fn and<F>(self, other: F) -> And<Self, F>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Success: Extract<T>,
        <<<Self::Output as IntoOutcome<T>>::Success as Extract<T>>::Extract as Tuple>::HList:
            Combine<
                <<<F::Output as IntoOutcome<T>>::Success as Extract<T>>::Extract as Tuple>::HList,
            >,
        F: Service<T> + Clone,
        <F::Output as IntoOutcome<T>>::Success: Extract<T>,
    {
        And::new(self, other)
    }

    fn map<F>(self, fun: F) -> Map<Self, F>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Success: Extract<T>,
        F: Func<<<Self::Output as IntoOutcome<T>>::Success as Extract<T>>::Extract> + Clone,
    {
        Map::new(self, fun)
    }

    // Boxing

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn boxed(
        self,
    ) -> BoxService<
        'static,
        T,
        <Self::Output as IntoOutcome<T>>::Success,
        <Self::Output as IntoOutcome<T>>::Failure,
    >
    where
        Self: Sized + 'static + Send + Sync,
        Self::Future: 'static + Send,
    {
        Box::new(BoxedService::new(self))
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn boxed_local(
        self,
    ) -> LocalBoxService<
        'static,
        T,
        <Self::Output as IntoOutcome<T>>::Success,
        <Self::Output as IntoOutcome<T>>::Failure,
    >
    where
        Self: Sized + 'static,
        Self::Future: 'static,
    {
        Box::new(LocalBoxedService::new(self))
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn shared(self) -> crate::combinators::shared::SharedService<Self>
    where
        Self: Sized,
    {
        crate::combinators::shared::SharedService::new(self)
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn shared_local(self) -> crate::combinators::shared::LocalSharedService<Self>
    where
        Self: Sized,
    {
        crate::combinators::shared::LocalSharedService::new(self)
    }
}

impl<T, I> ServiceExt<I> for T where T: Service<I> {}
