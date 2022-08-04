use core::future::Future;

use crate::{
    and::And,
    err_into::ErrInto,
    generic::{Combine, Extract, Func, Tuple},
    into_outcome::IntoOutcome,
    map::Map,
    map_err::MapErr,
    middleware::{Middleware, MiddlewareFn, MiddlewareFnService},
    service::Service,
};

#[cfg(feature = "alloc")]
use crate::boxed::{Box, BoxService, BoxedService};

#[cfg(feature = "alloc")]
use core::convert::Infallible;

#[cfg(feature = "alloc")]
use crate::outcome::Outcome;

#[cfg(feature = "alloc")]
use either::Either;

use crate::or::OrService;

pub trait ServiceExt<T>: Service<T> {
    fn or<O: Service<T>>(self, service: O) -> OrService<Self, O, T>
    where
        Self: Sized,
    {
        OrService::new(self, service)
    }

    #[cfg(feature = "alloc")]
    fn then<F, O>(
        self,
        then: F,
    ) -> BoxService<
        'static,
        T,
        <O::Output as IntoOutcome<T>>::Success,
        <O::Output as IntoOutcome<T>>::Failure,
    >
    where
        Self: Sized + Clone + Send + Sync + 'static,
        <Self as Service<T>>::Output: Send,
        <<Self as Service<T>>::Output as IntoOutcome<T>>::Failure: Send,
        <<Self as Service<T>>::Output as IntoOutcome<T>>::Success: Send,
        T: Send + 'static,
        F: Fn(Self::Output) -> O + Clone + Send + Sync + 'static,

        O: Future + Send + 'static,
        O::Output: IntoOutcome<T>,
    {
        (move |req: T| {
            let this = self.clone();
            let then = then.clone();
            async move {
                let ret = this.call(req).await;
                then(ret).await
            }
        })
        .boxed()
    }

    #[cfg(feature = "alloc")]
    fn and_then<F, O>(
        self,
        then: F,
    ) -> BoxService<'static, T, O::Output, <Self::Output as IntoOutcome<T>>::Failure>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        <Self as Service<T>>::Output: Send,
        <<Self as Service<T>>::Output as IntoOutcome<T>>::Failure: Send,
        <<Self as Service<T>>::Output as IntoOutcome<T>>::Success: Send,
        T: Send + 'static,
        F: Fn(<Self::Output as IntoOutcome<T>>::Success) -> O + Clone + Send + Sync + 'static,

        O: Future + Send + 'static,
    {
        (move |req: T| {
            let this = self.clone();
            let then = then.clone();
            async move {
                match this.call(req).await.into_outcome() {
                    Outcome::Failure(err) => Outcome::Failure(err),
                    Outcome::Next(next) => Outcome::Next(next),
                    Outcome::Success(ret) => Outcome::Success(then(ret).await),
                }
            }
        })
        .boxed()
    }

    #[cfg(feature = "alloc")]
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
        U: Send + Future,
        U::Output: IntoOutcome<T>,
    {
        self.wrap(MiddlewareFn::new(middleware))
    }

    #[cfg(feature = "alloc")]
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
        Self::Future: 'static,
    {
        Box::new(BoxedService::new(self))
    }

    fn map_err<F, E>(self, func: F) -> MapErr<F, Self, E>
    where
        Self: Sized,
        F: Fn(<Self::Output as IntoOutcome<T>>::Failure) -> E + Send + Clone,
    {
        MapErr::new(self, func)
    }

    fn err_into<E>(self) -> ErrInto<Self, E>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Failure: Into<E> + Send,
    {
        ErrInto::new(self)
    }

    #[cfg(any(feature = "alloc", feature = "std"))]
    fn shared(self) -> crate::shared::SharedService<Self>
    where
        Self: Sized,
    {
        crate::shared::SharedService::new(self)
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
        And {
            first: self,
            second: other,
        }
    }

    fn map<F>(self, fun: F) -> Map<Self, F>
    where
        Self: Sized,
        <Self::Output as IntoOutcome<T>>::Success: Extract<T>,
        F: Func<<<Self::Output as IntoOutcome<T>>::Success as Extract<T>>::Extract> + Clone,
    {
        Map {
            filter: self,
            callback: fun,
        }
    }
}

impl<T, I> ServiceExt<I> for T where T: Service<I> {}
