use crate::error::Error;
use crate::types::{IntoRequest, Response, ResponseWithBody, Transport};
use async_trait::async_trait;
use bytes::Bytes;
use mime::Mime;
use reqwest::{Request as HttpRequest, Response as HttpResponse};
use std::path::Path;

use super::Http;
use headers::{ContentType, HeaderMapExt};

#[async_trait]
impl Response for HttpResponse {
    type Error = reqwest::Error;
    async fn write_to(self, path: &Path) -> Result<(), Self::Error> {
        tokio::fs::write(path, self.bytes().await?)
            .await
            .expect("fs");
        Ok(())
    }
}

#[async_trait]
impl ResponseWithBody for HttpResponse {
    fn len(&self) -> Option<usize> {
        self.content_length().map(|m| m as usize)
    }
    fn mime(&self) -> Mime {
        let mime = self
            .headers()
            .typed_get::<ContentType>()
            .map(|m| m.into())
            .unwrap_or_else(|| mime::APPLICATION_OCTET_STREAM);

        mime
    }
    async fn body(mut self) -> Result<Bytes, reqwest::Error> {
        self.bytes().await
    }
}

#[async_trait::async_trait]
impl Transport for Http {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = Error<reqwest::Error>;

    async fn fetch<R>(&self, req: R) -> Result<Self::Response, Self::Error>
    where
        R: IntoRequest<Self>,
        R::Error: 'static,
    {
        let req = req.into_request().map_err(Error::request)?;

        let resp = self.client.execute(req).await?;

        Ok(resp)
    }
}
