use bytes::Bytes;
use futures_core::{ready, Stream};
use http_body::Body;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};

#[pin_project]
#[derive(Debug)]
pub struct BodyStream<B: Body>(#[pin] B);

impl<B: Body> BodyStream<B>
where
    B::Data: Into<Bytes>,
{
    pub fn new(body: B) -> BodyStream<B> {
        BodyStream(body)
    }
}

impl<B: Body> Stream for BodyStream<B>
where
    B::Data: Into<Bytes>,
{
    type Item = Result<Bytes, B::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match ready!(Body::poll_data(this.0, cx)) {
            Some(Ok(ret)) => Poll::Ready(Some(Ok(ret.into()))),
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}
