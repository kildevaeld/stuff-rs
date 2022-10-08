use std::{fs::File, path::PathBuf};

use dale::{Outcome, Service, ServiceExt};
use futures_core::Future;
use http::Request;

use crate::error::Error;

use self::file_options::FileOptions;

mod file_conditional;
mod file_options;

pub fn file<P, B>(
    path: P,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
>
where
    P: Into<PathBuf>,
    B: Send + 'static,
{
    dale::filters::state(path.into())
        .and(file_options::file_options())
        .and_then(
            |(req, (path, options)): (Request<B>, (PathBuf, FileOptions))| async move {
                //
                Result::<_, Error>::Ok((req, ()))
            },
        )
        .err_into()
}
