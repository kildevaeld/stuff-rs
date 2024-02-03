use std::path::Path;

use async_trait::async_trait;
use bytes::{Buf, Bytes};
use mime::Mime;

use crate::BoxError;

pub trait IntoRequest<F>: Send
where
    F: Transport,
{
    type Error: std::error::Error + Send + Sync;
    fn into_request(self) -> Result<F::Request, Self::Error>;
}

#[async_trait]
pub trait Response {
    type Error;

    async fn write_to(self, path: &Path) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait ResponseWithBody: Response {
    fn len(&self) -> Option<usize>;
    fn mime(&self) -> Mime;
    async fn body(self) -> Result<Bytes, Self::Error>;
}

#[async_trait]
pub trait Transport: Sized {
    type Request;
    type Response;
    type Error;

    async fn fetch<R>(&self, req: R) -> Result<Self::Response, Self::Error>
    where
        R: IntoRequest<Self>,
        R::Error: 'static;
}

#[async_trait]
pub trait ResponseBox: Send + Sync {
    fn len(&self) -> Option<usize>;
    fn mime(&self) -> Mime;

    async fn body(self: Box<Self>) -> Result<Bytes, BoxError>;
    async fn write_to(self: Box<Self>, path: &Path) -> Result<(), BoxError>;
}

pub struct BoxedResponse<R> {
    i: R,
}

#[async_trait]
impl<R> ResponseBox for BoxedResponse<R>
where
    R: ResponseWithBody + Send + Sync,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    fn len(&self) -> Option<usize> {
        self.i.len()
    }
    fn mime(&self) -> Mime {
        self.i.mime()
    }
    async fn body(self: Box<Self>) -> Result<Bytes, BoxError> {
        let bytes = self.i.body().await.map_err(Box::new)?;
        Ok(bytes)
    }
    async fn write_to(self: Box<Self>, path: &Path) -> Result<(), BoxError> {
        Ok(self.i.write_to(path).await.map_err(Box::new)?)
    }
}

pub fn response_box<R>(resp: R) -> Box<dyn ResponseBox>
where
    R: ResponseWithBody + Send + Sync + 'static,
    R::Error: std::error::Error + Send + Sync + 'static,
{
    Box::new(BoxedResponse { i: resp })
}
