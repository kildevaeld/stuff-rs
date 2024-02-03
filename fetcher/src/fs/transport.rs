use crate::{
    error::Error,
    types::{IntoRequest, Response, ResponseWithBody, Transport},
};
use async_trait::async_trait;
use bytes::Bytes;
use mime::Mime;
use relative_path::RelativePathBuf;
use std::{
    io,
    path::{Path, PathBuf},
};
use tokio::io::AsyncReadExt;

use super::File;

pub struct FileResponse {
    file: tokio::fs::File,
    size: usize,
    mime: Mime,
}

#[async_trait]
impl Response for FileResponse {
    type Error = std::io::Error;
    async fn write_to(mut self, path: &Path) -> Result<(), Self::Error> {
        let mut dest = tokio::fs::OpenOptions::default()
            .write(true)
            .create_new(true)
            .open(path)
            .await?;

        tokio::io::copy(&mut self.file, &mut dest).await?;

        Ok(())
    }
}

#[async_trait]
impl ResponseWithBody for FileResponse {
    fn len(&self) -> Option<usize> {
        self.size.into()
    }
    fn mime(&self) -> Mime {
        self.mime.clone()
    }

    async fn body(mut self) -> Result<Bytes, io::Error> {
        let mut buf = Vec::default();
        self.file.read_to_end(&mut buf).await?;
        Ok(buf.into())
    }
}

#[async_trait]
impl Transport for File {
    type Request = RelativePathBuf;
    type Response = FileResponse;
    type Error = Error<io::Error>;

    async fn fetch<R>(&self, req: R) -> Result<Self::Response, Self::Error>
    where
        R: IntoRequest<Self>,
        R::Error: 'static,
    {
        let req = req.into_request().map_err(Error::request)?;

        let path = req.to_logical_path(&self.root);

        let file = tokio::fs::OpenOptions::default()
            .read(true)
            .open(req.to_logical_path(&self.root))
            .await?;

        let meta = file.metadata().await?;

        let mime = mime_guess::from_path(&path).first_or_octet_stream();

        Ok(FileResponse {
            file,
            mime,
            size: meta.len() as usize,
        })
    }
}
