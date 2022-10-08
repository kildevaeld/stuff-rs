#[cfg(feature = "headers")]
use crate::error::KnownError;
use crate::{common::ToText, error::Error};
use bytes::{Buf, Bytes};
use dale::{Outcome, Service, ServiceExt};
use futures_core::Future;
#[cfg(feature = "headers")]
use headers::ContentLength;
use http::Request;
// #[cfg(feature = "headers")]
use crate::Body;

use std::{convert::Infallible, fmt};

#[derive(Debug)]
pub struct BodyReadError<E>(pub E);

impl<E> fmt::Display for BodyReadError<E>
where
    E: std::error::Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "body read error: {}", self.0)
    }
}

impl<E> std::error::Error for BodyReadError<E> where E: std::error::Error {}

#[cfg(feature = "headers")]
pub fn content_length_limit<B: Send + 'static>(
    limit: u64,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
> + Copy {
    crate::filters::header::header()
        .and_then(
            move |(req, (ContentLength(length),)): (Request<B>, (ContentLength,))| async move {
                if length <= limit {
                    Ok((req, ()))
                } else {
                    log::debug!("content-length: {} is over limit {}", length, limit);
                    Err(Error::from(KnownError::PayloadTooLarge))
                }
            },
        )
        .err_into()
}

pub fn body<B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (B,)), Infallible, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    |mut req: Request<B>| async move {
        let body = std::mem::replace(req.body_mut(), B::empty());
        Outcome::Success((req, (body,)))
    }
}

pub fn aggregate<B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (impl Buf,)), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    body()
        .and_then(|(req, (body,))| async move {
            match crate::common::Aggregate::new(body).await {
                Ok(ret) => Ok((req, (ret,))),
                Err(err) => Err(err),
            }
        })
        .err_into()
}

pub fn bytes<B: Body + Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (Bytes,)), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    body()
        .and_then(|(req, (body,))| async move {
            match crate::common::ToBytes::new(body).await {
                Ok(ret) => Ok((req, (ret,))),
                Err(err) => Err(err),
            }
        })
        .err_into()
}

pub fn text<B: Body + Send + 'static + Default>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, (String,)), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy
where
    B::Error: std::error::Error + Send + Sync,
    B::Data: Send,
{
    |mut req: Request<B>| async move {
        let body = dale::fail!(ToText::new(&mut req).await);
        dale::success!((req, (body,)))
    }
}
