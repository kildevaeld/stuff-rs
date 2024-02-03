use std::convert::Infallible;

use crate::{
    response_box,
    service::{Request, Response},
    BoxError, Http, IntoRequest, ResponseWithBody, Transport,
};
use dale::{boxed::BoxFuture, IntoService, Outcome, Service};
use reqwest::header::HeaderName;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum HttpRequestError {
    #[error("invalid header name: {0}")]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
}

impl IntoRequest<Http> for Request {
    type Error = HttpRequestError;

    fn into_request(self) -> Result<<Http as crate::Transport>::Request, Self::Error> {
        let mut req = reqwest::Request::new(reqwest::Method::GET, self.url);

        if let Some(meta) = self.meta {
            let headers = req.headers_mut();
            for (k, v) in meta.into_iter() {
                headers.append(HeaderName::from_bytes(k.as_bytes())?, v.parse()?);
            }
        }

        Ok(req)
    }
}

fn is_http(url: &url::Url) -> bool {
    match url.scheme() {
        "http" | "https" => true,
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub struct HttpService {
    client: Http,
}

impl Service<Request> for HttpService {
    type Output = Outcome<Response, BoxError, Request>;
    type Future = BoxFuture<'static, Self::Output>;

    fn call(&self, req: Request) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            //
            if !is_http(req.url()) {
                return Outcome::Next(req);
            }

            match client.fetch(req).await {
                Ok(ret) => Outcome::<_, _, Request>::Success(Response {
                    meta: Default::default(),
                    mime: ret.mime(),
                    len: ret.len(),
                    resp: Some(response_box(ret)),
                }),
                Err(err) => Outcome::Failure(Box::new(err) as BoxError),
            }
        })
    }
}

impl IntoService<Request> for Http {
    type Error = Infallible;
    type Service = HttpService;

    fn into_service(self) -> Result<Self::Service, Self::Error> {
        Ok(HttpService { client: self })
    }
}
