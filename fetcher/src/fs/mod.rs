use std::path::PathBuf;

mod request;
mod service;
mod transport;

#[derive(Debug, Clone)]
pub struct File {
    root: PathBuf,
}

impl File {
    pub fn new(dir: impl Into<PathBuf>) -> File {
        File {
            root: dir.into().canonicalize().expect("path does not exists"),
        }
    }
}
