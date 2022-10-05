use std::path::{Path, PathBuf};

pub trait Request {}

pub trait IntoRequest<F>
where
    F: Fetcher,
{
    fn into_request(self) -> Result<F::Request, F::Error>;
}

pub trait Response {
    fn write_to(self, path: &Path);
}

pub trait ResponseWithBytes: Response {
    type Body;
    fn body(&self) -> Option<Self::Body>;
}

pub trait Fetcher {
    type Request;
    type Response;
    type Error;

    fn fetch(&self, req: Self::Request) -> Result<Self::Response, Self::Error>;
}
