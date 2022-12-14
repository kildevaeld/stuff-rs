use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt;

use dale::Either;

use crate::filters::BodyReadError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    error: Box<dyn StdError + Send + Sync>,
}

impl Error {
    pub fn new<E>(error: E) -> Error
    where
        E: Into<Box<dyn StdError + Send + Sync>>,
    {
        Error {
            error: error.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&dyn StdError> {
        Some(&*self.error)
    }
}

impl From<Infallible> for Error {
    fn from(error: Infallible) -> Error {
        Error {
            error: Box::new(KnownError::Internal(Box::new(error))),
        }
    }
}

impl From<KnownError> for Error {
    fn from(error: KnownError) -> Error {
        Error {
            error: Box::new(error),
        }
    }
}

#[cfg(feature = "hyper")]
impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Error {
        Error {
            error: Box::new(KnownError::Internal(Box::new(error))),
        }
    }
}

impl<E: StdError + Send + Sync + 'static> From<BodyReadError<E>> for Error {
    fn from(error: BodyReadError<E>) -> Error {
        Error {
            error: Box::new(KnownError::Internal(Box::new(error))),
        }
    }
}

impl<L, R> From<Either<L, R>> for Error
where
    L: Into<Error>,
    R: Into<Error>,
{
    fn from(err: Either<L, R>) -> Self {
        match err {
            Either::Left(left) => left.into(),
            Either::Right(right) => right.into(),
        }
    }
}

#[derive(Debug)]
pub enum KnownError {
    Internal(Box<dyn StdError + Send + Sync>),
    PayloadTooLarge,
    InvalidHeader(String),
    MissingHeader(String),
    Utf8(std::str::Utf8Error),
}

impl fmt::Display for KnownError {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl StdError for KnownError {}
