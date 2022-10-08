mod body;
pub mod common;
pub mod error;
// mod file;
pub mod filters;
mod modifier;
mod modifiers;
pub mod reply;
mod request_ext;

#[cfg(feature = "hyper")]
pub mod hyper;

pub use http::{Method, Request, Response, StatusCode, Uri};

pub use self::{body::Body, error::Result, reply::Reply};

pub type Outcome<B> = dale::Outcome<Response<B>, error::Error, Request<B>>;

pub mod prelude {
    pub use super::{modifier::*, request_ext::*};
    pub use dale::{IntoOutcomeExt, ServiceExt};
}

//
mod taker;
pub use self::taker::*;
