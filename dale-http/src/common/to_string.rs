use super::ToBytes;
use crate::{error::Error, Body};
use futures_core::{ready, Future};
use http::Request;
use pin_project_lite::pin_project;
use std::task::Poll;

pin_project! {
    pub struct ToText<'a, B> where B: Body {
        #[pin]
        state: State<'a, B>
    }
}

impl<'a, B> ToText<'a, B>
where
    B: Body,
{
    pub fn new(req: &'a mut Request<B>) -> ToText<'a, B> {
        ToText {
            state: State::First { req },
        }
    }
}

pin_project! {
    #[project = StateProj]
    enum State<'a, B> where B: Body {
        First { req: &'a mut Request<B> },
        Second {
            #[pin]
            future: ToBytes<B>,
        }
    }
}

impl<'a, B> Future for ToText<'a, B>
where
    B: Body,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    type Output = Result<String, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            let state = match this.state.project() {
                StateProj::First { req } => {
                    let body = std::mem::replace(req.body_mut(), B::empty());
                    State::Second {
                        future: ToBytes::new(body),
                    }
                }
                StateProj::Second { future } => {
                    let bytes = match ready!(future.poll(cx)) {
                        Ok(bytes) => bytes,
                        Err(err) => return Poll::Ready(Err(Error::new(err))),
                    };

                    let text = match String::from_utf8(bytes.to_vec()) {
                        Ok(ret) => ret,
                        Err(err) => panic!("not valid string"),
                    };

                    return Poll::Ready(Ok(text));
                }
            };

            self.set(ToText { state })
        }
    }
}
