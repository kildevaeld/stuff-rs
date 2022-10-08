use dale::{filters::One, Outcome, Service, ServiceExt};
#[cfg(feature = "headers")]
use headers::{Header, HeaderMapExt};
use http::{
    header::{AsHeaderName, HeaderValue},
    Request,
};
use std::future::Future;

use crate::error::{Error, KnownError};

#[cfg(feature = "headers")]
pub fn header<H: Header + Send + 'static, B: Send + 'static>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, One<H>), Error, Request<B>>,
> + Copy {
    move |req: Request<B>| {
        tracing::trace!("header({:?})", H::name());
        async move {
            let header = match req.headers().typed_get() {
                Some(header) => header,
                None => {
                    return Outcome::Failure(
                        KnownError::MissingHeader(H::name().as_str().to_owned()).into(),
                    )
                }
            };

            Outcome::Success((req, (header,)))
        }
    }
}

pub fn header_str<S: AsHeaderName + Clone + Send + Sync + 'static, B: Send + 'static>(
    name: S,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, One<HeaderValue>), Error, Request<B>>,
> + Clone {
    move |req: Request<B>| {
        let name = name.clone();

        async move {
            match req.headers().get(name.clone()).map(|h| h.clone()) {
                Some(s) => Outcome::Success((req, (s,))),
                None => Outcome::Failure(Error::from(KnownError::InvalidHeader(
                    name.as_str().to_owned(),
                ))),
            }
        }
    }
}

#[cfg(feature = "headers")]
pub fn optional<H: Header + Send + 'static, B: Send + 'static>() -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, One<Option<H>>), Error, Request<B>>,
> + Copy {
    |req: Request<B>| async move {
        let header = req.headers().typed_get();
        Outcome::Success((req, (header,)))
    }
}

// #[cfg(feature = "headers")]
// pub fn optional<H: Header, B>() -> Optional<B, H> {
//     Optional(PhantomData)
// }

// #[derive(Debug)]
// pub struct Optional<B, H: Header>(PhantomData<(B, H)>);

// impl<B, H: Header> Clone for Optional<B, H> {
//     fn clone(&self) -> Self {
//         Optional(PhantomData)
//     }
// }

// impl<B, H: Header> Copy for Optional<B, H> {}

// unsafe impl<B, H: Header> Send for Optional<B, H> {}

// unsafe impl<B, H: Header> Sync for Optional<B, H> {}

// impl<B, H: Header> Service<Request<B>> for Optional<B, H> {
//     type Output = Outcome<(Request<B>, One<Option<H>>), Error, Request<B>>;

//     type Future = std::future::Ready<Self::Output>;

//     fn call(&self, req: Request<B>) -> Self::Future {
//         let header = req.headers().typed_get();
//         std::future::ready(Outcome::Success((req, (header,))))
//     }
// }

pub fn optional_str<S: AsHeaderName + Clone + Send + Sync + 'static, B: Send + 'static>(
    name: S,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, One<Option<HeaderValue>>), Error, Request<B>>,
> + Clone {
    dale::filters::state(name)
        .and_then(|req: (Request<B>, (S,))| async move {
            let (req, (name,)) = req;
            let header = req.headers().get(name).map(|h| h.clone());
            Result::<_, Error>::Ok((req, (header,)))
        })
        .err_into()
}

pub fn exact<B: Send + 'static>(
    name: &'static str,
    value: &'static str,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
> + Copy {
    move |req: Request<B>| async move {
        tracing::trace!("exact?({:?}, {:?})", name, value);

        let val = match req.headers().get(name) {
            Some(header) => header,
            None => return Outcome::Failure(KnownError::MissingHeader(name.to_owned()).into()),
        };

        if val != value {
            return Outcome::Failure(KnownError::InvalidHeader(name.to_owned()).into());
        }

        Outcome::Success((req, ()))
    }
}

/// Create a `Filter` that requires a header to match the value exactly.
///
/// This `Filter` will look for a header with supplied name and the exact
/// value, ignoring ASCII case, otherwise rejects the request.
///
/// # Example
///
/// ```
/// // Require `connection: keep-alive` header to be set.
/// let keep_alive = veep_core::filters::header::exact("connection", "keep-alive");
/// ```
pub fn exact_ignore_case<B: Send + 'static>(
    name: &'static str,
    value: &'static str,
) -> impl Service<
    Request<B>,
    Future = impl Future + Send,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
> + Copy {
    move |req: Request<B>| async move {
        tracing::trace!("exact_ignore_case({:?}, {:?})", name, value);

        let val = match req.headers().get(name) {
            Some(s) => s,
            None => return Outcome::Failure(KnownError::MissingHeader(name.to_owned()).into()),
        };

        let ret = if val.as_bytes().eq_ignore_ascii_case(value.as_bytes()) {
            Outcome::Success((req, ()))
        } else {
            Outcome::Failure(KnownError::InvalidHeader(name.to_owned()).into())
        };

        ret
    }
}
