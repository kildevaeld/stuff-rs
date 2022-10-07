use crate::body::Body;

// use bytes::Bytes;
use crate::modifier::Modifier;
use http::{header, Response, StatusCode, Uri};

impl<B: Body> Modifier<Response<B>> for Vec<u8> {
    #[inline]
    fn modify(self, res: &mut Response<B>) {
        let len = self.len();
        *res.body_mut() = B::from_bytes(self);
        res.headers_mut()
            .insert(header::CONTENT_LENGTH, (len as u64).into());
    }
}

impl<B: Body> Modifier<Response<B>> for String {
    #[inline]
    fn modify(self, res: &mut Response<B>) {
        self.into_bytes().modify(res)
    }
}

impl<'a, B: Body> Modifier<Response<B>> for &'a str {
    #[inline]
    fn modify(self, res: &mut Response<B>) {
        self.to_owned().modify(res);
    }
}

impl<'a, B: Body> Modifier<Response<B>> for &'a [u8] {
    #[inline]
    fn modify(self, res: &mut Response<B>) {
        self.to_vec().modify(res);
    }
}

impl<B: Body> Modifier<Response<B>> for bytes::Bytes {
    #[inline]
    fn modify(self, res: &mut Response<B>) {
        self.to_vec().modify(res)
    }
}

impl<B> Modifier<Response<B>> for StatusCode {
    fn modify(self, res: &mut Response<B>) {
        *res.status_mut() = self;
    }
}

impl<B> Modifier<Response<B>> for header::HeaderMap {
    #[inline]
    fn modify(self, res: &mut Response<B>) {
        res.headers_mut().extend(self)
    }
}

/// A modifier for changing headers on requests and responses.
#[derive(Clone)]
pub struct Header<H>(pub H, pub header::HeaderValue);

impl<B, H> Modifier<Response<B>> for Header<H>
where
    H: header::IntoHeaderName,
{
    fn modify(self, res: &mut Response<B>) {
        res.headers_mut().insert(self.0, self.1);
    }
}

#[cfg(feature = "headers")]
mod headers_impl {

    use super::*;
    pub use headers::{
        AcceptRanges, Allow, CacheControl, Connection, ContentLength, ContentRange, ContentType,
        ETag, Expires, HeaderMap, HeaderMapExt, LastModified, Location, SecWebsocketAccept,
        Upgrade, UserAgent,
    };
    macro_rules! typed_header_impl {
        ($header: ty) => {
            impl<B> Modifier<Response<B>> for $header {
                fn modify(self, res: &mut Response<B>) {
                    res.headers_mut().typed_insert(self)
                }
            }
        };
    }
    typed_header_impl!(ContentType);
    typed_header_impl!(ContentLength);
    typed_header_impl!(CacheControl);
    typed_header_impl!(ETag);
    typed_header_impl!(Expires);
    typed_header_impl!(Upgrade);
    typed_header_impl!(Connection);
    typed_header_impl!(SecWebsocketAccept);
    typed_header_impl!(ContentRange);
    typed_header_impl!(AcceptRanges);
    typed_header_impl!(LastModified);
    typed_header_impl!(Allow);
}

/// A modifier for creating redirect responses.
pub struct Redirect(pub Uri);

impl<B> Modifier<Response<B>> for Redirect {
    fn modify(self, res: &mut Response<B>) {
        let Redirect(url) = self;
        // Url should always be parsable to a valid HeaderValue, so unwrap should be safe here.
        res.headers_mut()
            .insert(header::LOCATION, url.to_string().parse().unwrap());
    }
}

// /// A modifier for creating redirect responses.
// pub struct RedirectRaw(pub Uri);

// impl Modifier<Response> for RedirectRaw {
//     fn modify(self, res: &mut Response) {
//         let RedirectRaw(path) = self;
//         res.headers.insert(header::LOCATION, path.parse().unwrap());
//     }
// }

impl<B, F> Modifier<Response<B>> for F
where
    F: Fn(&mut Response<B>),
{
    fn modify(self, res: &mut Response<B>) {
        (self)(res)
    }
}
