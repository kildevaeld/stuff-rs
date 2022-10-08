use crate::into_outcome::IntoOutcome;
use core::future::Future;

pub trait Service<T> {
    type Output: IntoOutcome<T>;
    type Future: Future<Output = Self::Output>;
    fn call(&self, req: T) -> Self::Future;
}

impl<T, F, U> Service<T> for F
where
    F: Fn(T) -> U,
    U: Future,
    U::Output: IntoOutcome<T>,
{
    type Output = U::Output;

    type Future = U;

    fn call(&self, req: T) -> Self::Future {
        (self)(req)
    }
}

pub struct ServiceFn<F> {
    func: F,
}

impl<T, F, U> Service<T> for ServiceFn<F>
where
    F: Fn(T) -> U,
    U: Future,
    U::Output: IntoOutcome<T>,
{
    type Output = U::Output;

    type Future = U;

    fn call(&self, req: T) -> Self::Future {
        (self.func)(req)
    }
}

pub fn service<F>(service: F) -> ServiceFn<F> {
    ServiceFn { func: service }
}
