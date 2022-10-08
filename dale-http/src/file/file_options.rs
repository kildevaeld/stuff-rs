use std::convert::Infallible;

use dale::{IntoOutcome, Outcome, Service, ServiceExt};
use futures_core::Future;
use headers::{IfModifiedSince, IfRange, IfUnmodifiedSince, LastModified, Range};
use http::{Request, Response, StatusCode};

use crate::{
    error::Error,
    filters,
    modifier::{Modifier, With},
    Body,
};

#[derive(Debug)]
pub(crate) struct FileOptions {
    if_modified_since: Option<IfModifiedSince>,
    if_unmodified_since: Option<IfUnmodifiedSince>,
    if_range: Option<IfRange>,
    range: Option<Range>,
}

// impl<N> IntoOutcome<N> for FileOptions {
//     type Success = Self;

//     type Failure = Infallible;

//     fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
//         Outcome::Success(self)
//     }
// }

pub(crate) enum Cond<B> {
    NoBody(Response<B>),
    WithBody(Option<Range>),
}

impl FileOptions {
    pub fn check<B>(self, last_modified: Option<LastModified>) -> Cond<B>
    where
        B: Body + Modifier<Response<B>>,
    {
        if let Some(since) = self.if_unmodified_since {
            let precondition = last_modified
                .map(|time| since.precondition_passes(time.into()))
                .unwrap_or(false);

            log::trace!(
                "if-unmodified-since? {:?} vs {:?} = {}",
                since,
                last_modified,
                precondition
            );
            if !precondition {
                let res = Response::with(StatusCode::PRECONDITION_FAILED);
                return Cond::NoBody(res);
            }
        }

        if let Some(since) = self.if_modified_since {
            log::trace!(
                "if-modified-since? header = {:?}, file = {:?}",
                since,
                last_modified
            );
            let unmodified = last_modified
                .map(|time| !since.is_modified(time.into()))
                // no last_modified means its always modified
                .unwrap_or(false);
            if unmodified {
                let res = Response::with(StatusCode::NOT_MODIFIED);
                return Cond::NoBody(res);
            }
        }

        if let Some(if_range) = self.if_range {
            log::trace!("if-range? {:?} vs {:?}", if_range, last_modified);
            let can_range = !if_range.is_modified(None, last_modified.as_ref());

            if !can_range {
                return Cond::WithBody(None);
            }
        }

        Cond::WithBody(self.range)
    }
}

pub(crate) fn file_options<B>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    // Output = Outcome<(Request<B>, ()), Error, Request<B>>,
>
where
    B: Send + 'static,
{
    // filters::header::optional()
    //     .and(filters::header::optional())
    //     .and(filters::header::optional())
    //     .and(filters::header::optional())
    //     .map(
    //         |if_modified_since, if_unmodified_since, if_range, range| FileOptions {
    //             if_modified_since,
    //             if_unmodified_since,
    //             if_range,
    //             range,
    //         },
    //     )
    filters::header::optional()
        .and(filters::header::optional())
        .map(
            |if_modified_since: Option<IfModifiedSince>,
             if_unmodified_since: Option<IfUnmodifiedSince>| {
                //
                ()
            },
        )
}
