use futures_core::{ready, Future};
use pin_project_lite::pin_project;
use std::task::Poll;

use crate::common::buf::BufList;
use http_body::Body;

pin_project! {
    /// Aggregate the data buffers from a body asynchronously.
    ///
    /// The returned `impl Buf` groups the `Buf`s from the `HttpBody` without
    /// copying them. This is ideal if you don't require a contiguous buffer.
    ///
    /// # Note
    ///
    /// Care needs to be taken if the remote is untrusted. The function doesn't implement any length
    /// checks and an malicious peer might make it consume arbitrary amounts of memory. Checking the
    /// `Content-Length` is a possibility, but it is not strictly mandated to be present.
    pub struct Aggregate<B> where B: Body {
        #[pin]
        body: B,
        bufs: Option<BufList<B::Data>>
    }
}

impl<B> Aggregate<B>
where
    B: Body,
{
    pub fn new(body: B) -> Aggregate<B> {
        Aggregate {
            body,
            bufs: Some(BufList::new()),
        }
    }
}

impl<B> Future for Aggregate<B>
where
    B: Body,
{
    type Output = Result<BufList<B::Data>, B::Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            match ready!(this.body.poll_data(cx)) {
                Some(ret) => {
                    let ret = match ret {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    this.bufs.as_mut().unwrap().push(ret);
                }
                None => {
                    let bufs = this.bufs.take().unwrap();
                    return Poll::Ready(Ok(bufs));
                }
            }
        }
    }
}
