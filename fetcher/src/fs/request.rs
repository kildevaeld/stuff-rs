use std::{
    io,
    path::{Path, PathBuf},
};

use relative_path::RelativePathBuf;

use crate::types::IntoRequest;

use super::File;

// impl IntoRequest<File> for PathBuf {
//     type Error = io::Error;
//     fn into_request(self) -> Result<PathBuf, io::Error> {
//         Ok(self)
//     }
// }

// impl<'a> IntoRequest<File> for &'a Path {
//     type Error = io::Error;
//     fn into_request(self) -> Result<PathBuf, io::Error> {
//         Ok(self.to_path_buf())
//     }
// }

impl<'a> IntoRequest<File> for &'a str {
    type Error = io::Error;
    fn into_request(self) -> Result<RelativePathBuf, io::Error> {
        Ok(RelativePathBuf::from(self))
    }
}

#[derive(Debug)]
pub struct NotFilePath;

impl std::fmt::Display for NotFilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "url not a path")
    }
}

impl std::error::Error for NotFilePath {}

impl IntoRequest<File> for url::Url {
    type Error = NotFilePath;

    fn into_request(self) -> Result<<File as crate::Transport>::Request, Self::Error> {
        if self.scheme() != "file" {
            return Err(NotFilePath);
        }

        let path = if let Some(host) = self.host_str() {
            RelativePathBuf::from(format!("{}{}", host, self.path()))
        } else {
            RelativePathBuf::from(self.path())
        };

        Ok(path)
    }
}
