use std::future::Future;

use crate::error::Error;
use dale::{filters::One, Outcome, Service};
use http::{Method, Request};

pub fn get<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    method_is::<_, B>(|| &Method::GET)
}

pub fn post<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    method_is::<_, B>(|| &Method::POST)
}

pub fn put<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    method_is::<_, B>(|| &Method::PUT)
}

pub fn patch<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    method_is::<_, B>(|| &Method::PATCH)
}

pub fn delete<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    method_is::<_, B>(|| &Method::DELETE)
}

pub fn options<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    method_is::<_, B>(|| &Method::OPTIONS)
}
pub fn method<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, One<Method>), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    |req: Request<B>| async move {
        let method = req.method().clone();
        Outcome::Success((req, (method,)))
    }
}

pub fn method_is<F, B: Send + 'static>(
    func: F,
) -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy
where
    F: 'static + Send + Fn() -> &'static Method + Copy,
{
    move |req: Request<B>| async move {
        let method = func();
        log::trace!("method::{:?}?: {:?}", method, req.method());
        if req.method() == method {
            Outcome::Success((req, ()))
        } else {
            Outcome::Next(req)
        }
    }
}
