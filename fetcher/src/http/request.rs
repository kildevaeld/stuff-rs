use super::Http;
use crate::types::IntoRequest;
use reqwest::{Body, Error as HttpError, Method, Request as HttpRequest, Url};
use thiserror::Error as ThisError;

impl IntoRequest<Http> for HttpRequest {
    type Error = HttpError;
    fn into_request(self) -> Result<HttpRequest, HttpError> {
        Ok(self)
    }
}

impl IntoRequest<Http> for Url {
    type Error = HttpError;
    fn into_request(self) -> Result<HttpRequest, HttpError> {
        Ok(HttpRequest::new(Method::GET, self))
    }
}

#[derive(Debug, ThisError)]
pub enum ParseError {
    #[error("url parse error: {0}")]
    Parse(#[from] url::ParseError),
    #[error("http error: {0}")]
    Http(#[from] HttpError),
}

impl<'a> IntoRequest<Http> for &'a str {
    type Error = ParseError;
    fn into_request(self) -> Result<HttpRequest, Self::Error> {
        let url: reqwest::Url = self.parse()?;
        let req = <Url as IntoRequest<Http>>::into_request(url)?;
        Ok(req)
    }
}

impl IntoRequest<Http> for String {
    type Error = ParseError;
    fn into_request(self) -> Result<HttpRequest, Self::Error> {
        <&str as IntoRequest<Http>>::into_request(self.as_str())
    }
}
