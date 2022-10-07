#[cfg(feature = "stream")]
use bytes::Bytes;
use dale::Service;
use dale::{IntoOutcome, Outcome};
use futures_core::ready;
#[cfg(feature = "stream")]
use futures_core::Stream;

use http::StatusCode;
use hyper::{service::Service as HyperService, Body, Request, Response};
use pin_project_lite::pin_project;
use std::{convert::Infallible, future::Future};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::reply::Reply;

pub struct MakeTaskHyperService<T> {
    task: T,
}

impl<T> MakeTaskHyperService<T> {
    pub fn new(task: T) -> MakeTaskHyperService<T> {
        MakeTaskHyperService { task: task }
    }
}

impl<'t, T, Ctx> HyperService<&'t Ctx> for MakeTaskHyperService<T>
where
    T: Send + Service<Request<Body>> + Clone + 'static,
    // <T::Output as IntoOutcome<Request<Body>>>::Success: Reply,
{
    type Response = DaisyHyperService<T>;
    type Error = Infallible;
    type Future = MakeTaskHyperServiceFuture<T>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _cx: &'t Ctx) -> Self::Future {
        MakeTaskHyperServiceFuture {
            service: Some(DaisyHyperService {
                service: self.task.clone(),
            }),
        }
    }
}

pin_project! {
    pub struct MakeTaskHyperServiceFuture<S> {
        service: Option<DaisyHyperService<S>>,
    }
}
impl<S> Future for MakeTaskHyperServiceFuture<S> {
    type Output = Result<DaisyHyperService<S>, Infallible>;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        Poll::Ready(Ok(this.service.take().unwrap()))
    }
}

pub struct DaisyHyperService<T> {
    service: T,
}

impl<T> HyperService<Request<Body>> for DaisyHyperService<T>
where
    T: Send + Service<Request<Body>> + Clone + 'static,
    <T::Output as IntoOutcome<Request<Body>>>::Success: Reply<Body>,
{
    type Response = Response<Body>;

    type Error = <T::Output as IntoOutcome<Request<Body>>>::Failure; //<TowerService<T> as HyperService<Request<Body>>>::Error;

    type Future = DaisyHyperServiceFuture<T>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let fut = self.service.call(req);
        DaisyHyperServiceFuture { future: fut }
    }
}

pin_project! {
    pub struct DaisyHyperServiceFuture<S> where S: Service<Request<Body>> {
        #[pin]
        future: S::Future,
    }
}
impl<S> Future for DaisyHyperServiceFuture<S>
where
    S: Service<Request<Body>>,
    <S::Output as IntoOutcome<Request<Body>>>::Success: Reply<Body>,
{
    type Output = Result<Response<Body>, <S::Output as IntoOutcome<Request<Body>>>::Failure>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let resp = match ready!(this.future.poll(cx)).into_outcome() {
            Outcome::Failure(err) => Err(err),
            Outcome::Next(_) => {
                let mut resp = Response::new(Body::from(StatusCode::NOT_FOUND.to_string()));
                *resp.status_mut() = StatusCode::NOT_FOUND;

                Ok(resp)
            }
            Outcome::Success(success) => Ok(success.into_response()),
        };

        Poll::Ready(resp)
    }
}

impl crate::body::Body for Body {
    fn empty() -> Self {
        Body::empty()
    }

    #[cfg(feature = "stream")]
    fn from_stream<S, O, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<O, E>> + Send + 'static,
        O: Into<Bytes> + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    {
        Body::wrap_stream(stream)
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        Body::from(bytes)
    }
}
