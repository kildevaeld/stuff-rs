use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Buf, BufMut, Bytes};

use futures_core::{ready, Future};
use http_body::Body;
use pin_project_lite::pin_project;

pin_project! {
    /// Concatenate the buffers from a body into a single `Bytes` asynchronously.
    ///
    /// This may require copying the data into a single buffer. If you don't need
    /// a contiguous buffer, prefer the [`aggregate`](crate::body::aggregate())
    /// function.
    ///
    /// # Note
    ///
    /// Care needs to be taken if the remote is untrusted. The function doesn't implement any length
    /// checks and an malicious peer might make it consume arbitrary amounts of memory. Checking the
    /// `Content-Length` is a possibility, but it is not strictly mandated to be present.
    pub struct ToBytes<B> where B: Body {
        #[pin]
        state: ToBytesState<B>
    }
}

impl<B> ToBytes<B>
where
    B: Body,
{
    pub fn new(body: B) -> ToBytes<B> {
        ToBytes {
            state: ToBytesState::First { body: Some(body) },
        }
    }
}

enum ToBytesState<B>
where
    B: Body,
{
    First {
        body: Option<B>,
    },
    Second {
        body: Option<B>,
        first: Option<B::Data>,
    },
    Rest {
        buf: Option<Vec<u8>>,
        body: B,
    },
    Done,
}

macro_rules! done {
    ($this: expr) => {
        $this.set(ToBytes {
            state: ToBytesState::Done,
        });
    };
    ($this: expr, $ret: expr) => {
        match $ret {
            Ok(ret) => ret,
            Err(err) => {
                done!($this);
                return Poll::Ready(Err(err));
            }
        }
    };
}

macro_rules! pin {
    ($expr: expr) => {
        unsafe { Pin::new_unchecked($expr) }
    };
}

impl<B> Future for ToBytes<B>
where
    B: Body,
{
    type Output = Result<Bytes, B::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            let state = unsafe { Pin::get_unchecked_mut(this.state) };

            let state = match state {
                ToBytesState::First { body } => {
                    //
                    match ready!(pin!(body.as_mut().unwrap()).poll_data(cx)) {
                        Some(data) => {
                            let ret = done!(self, data);
                            Some(ToBytesState::Second {
                                body: body.take(),
                                first: Some(ret),
                            })
                        }
                        None => {
                            done!(self);
                            return Poll::Ready(Ok(Bytes::new()));
                        }
                    }
                }
                ToBytesState::Second { body, first } => {
                    //
                    match ready!(pin!(body.as_mut().unwrap()).poll_data(cx)) {
                        Some(data) => {
                            let second = done!(self, data);

                            let first = first.take().unwrap();

                            let cap = first.remaining()
                                + second.remaining()
                                + body.as_ref().unwrap().size_hint().lower() as usize;
                            let mut buf = Vec::with_capacity(cap);
                            buf.put(first);
                            buf.put(second);

                            Some(ToBytesState::Rest {
                                body: body.take().unwrap(),
                                buf: Some(buf),
                            })
                        }
                        None => {
                            let mut first = first.take().unwrap();
                            let bytes = first.copy_to_bytes(first.remaining());
                            done!(self);
                            return Poll::Ready(Ok(bytes));
                        }
                    }
                }
                ToBytesState::Rest { buf, body } => match ready!(pin!(body).poll_data(cx)) {
                    Some(ret) => {
                        let ret = done!(self, ret);
                        buf.as_mut().unwrap().put(ret);
                        None
                    }
                    None => {
                        let buf = buf.take().unwrap();
                        done!(self);
                        return Poll::Ready(Ok(buf.into()));
                    }
                },
                ToBytesState::Done => panic!("poll after complete"),
            };

            if let Some(state) = state {
                self.set(ToBytes { state })
            }
        }
    }
}
