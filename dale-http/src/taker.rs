use std::future::Ready;

use bytes::Bytes;
use http::{Request, Uri};

use crate::{common::ToBytes, error::Error, Body};

pub trait Taker<T> {
    fn take(i: &mut T) -> Result<T, Error>;
}

pub trait FromRequest<'r, B> {
    type Future;
    fn from_request(req: &'r mut Request<B>) -> Self::Future;
}

impl<'r, B> FromRequest<'r, B> for &'r Uri {
    type Future = Ready<&'r Uri>;
    fn from_request(req: &'r mut Request<B>) -> Self::Future {
        std::future::ready(req.uri())
    }
}

impl<'r, B> FromRequest<'r, B> for Bytes
where
    B: Body,
{
    type Future = ToBytes<B>;
    fn from_request(req: &'r mut Request<B>) -> Self::Future {
        let body = std::mem::replace(req.body_mut(), B::empty());
        ToBytes::new(body)
    }
}
