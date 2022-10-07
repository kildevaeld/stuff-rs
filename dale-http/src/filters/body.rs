use crate::error::Error;
#[cfg(feature = "headers")]
use crate::KnownError;
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
    Output = (Request<B>, ()),
    Error = impl Into<Error> + Rejection<Request<B>>,
    Future = impl Future + Send,
> + Copy {
    use mdv::Reject;

    crate::filters::header().then(
        move |(req, (ContentLength(length),)): (Request<B>, (ContentLength,))| async move {
            if length <= limit {
                Ok((req, ()))
            } else {
                log::debug!("content-length: {} is over limit {}", length, limit);
                Err(Reject::Reject(
                    req,
                    Error::from(KnownError::PayloadTooLarge),
                ))
            }
        },
    )
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
            match crate::common::aggregate(body).await {
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
> + Clone
where
    B::Data: Send,
    B::Error: Into<Error> + Send,
{
    body()
        .and_then(|(req, (body,))| async move {
            match crate::common::to_bytes(body).await {
                Ok(ret) => Ok((req, (ret,))),
                Err(err) => Err(err),
            }
        })
        .err_into()
}

// pub fn text<B: Body + Send + 'static + Default>(
// ) -> impl Service<Request<B>, Output = (Request<B>, (String,)), Error = Error> + Copy
// where
//     B::Error: std::error::Error + Send + Sync,
//     B::Data: Send,
// {
//     bytes()
//         .and_then(|body| async move { do_text(body) })
//         .err_into()
// }

// fn do_text<B: Buf>(buf: B) -> Result<String, Error> {
//     let query = std::str::from_utf8(buf.chunk()).map_err(Error::new)?;
//     Ok(query.to_owned())
// }
