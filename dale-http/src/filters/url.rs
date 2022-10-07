use dale::{filters::One, Outcome, Service};
use futures_core::Future;
use http::{Request, Uri};

use crate::error::Error;

pub fn url<B: Send + 'static>() -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, One<Uri>), Error, Request<B>>,
    Future = impl Future + Send,
> + Copy {
    |req: Request<B>| async move {
        let uri = req.uri().clone();
        Outcome::Success((req, (uri,)))
    }
}

pub fn path<P: ToString, B: Send + 'static>(
    path: P,
) -> impl Service<
    Request<B>,
    Output = Outcome<(Request<B>, ()), Error, Request<B>>,
    Future = impl Future + Send,
> + Clone {
    let p = path.to_string();
    move |req: Request<B>| {
        let eql = req.uri().path() == &p;
        async move {
            if eql {
                Outcome::Success((req, ()))
            } else {
                Outcome::Next(req)
            }
        }
    }
}
