#[cfg(feature = "stream")]
use bytes::Bytes;
#[cfg(feature = "stream")]
use futures_core::Stream;
use http_body::Body as HttpBody;

pub trait Body: HttpBody + Sized {
    fn empty() -> Self;
    fn from_bytes(bytes: Vec<u8>) -> Self;

    #[cfg(feature = "stream")]
    fn from_stream<S, O, E>(stream: S) -> Self
    where
        S: Stream<Item = Result<O, E>> + Send + 'static,
        O: Into<Bytes> + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static;
}
