use crate::{
    response_box,
    service::{Request, Response},
    BoxError, File, IntoRequest, ResponseWithBody, Transport,
};
use dale::{boxed::BoxFuture, IntoService, Outcome, Service};
use std::convert::Infallible;

use super::request::NotFilePath;

impl IntoRequest<File> for Request {
    type Error = NotFilePath;

    fn into_request(self) -> Result<<File as crate::Transport>::Request, Self::Error> {
        <url::Url as IntoRequest<File>>::into_request(self.url)
    }
}

fn is_file(url: &url::Url) -> bool {
    url.scheme() == "file"
}

#[derive(Clone, Debug)]
pub struct FileService {
    client: File,
}

impl Service<Request> for FileService {
    type Output = Outcome<Response, BoxError, Request>;
    type Future = BoxFuture<'static, Self::Output>;

    fn call(&self, req: Request) -> Self::Future {
        let client = self.client.clone();
        Box::pin(async move {
            if !is_file(req.url()) {
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

impl IntoService<Request> for File {
    type Error = Infallible;
    type Service = FileService;

    fn into_service(self) -> Result<Self::Service, Self::Error> {
        Ok(FileService { client: self })
    }
}
