use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt;

use dale::Either;

use crate::filters::BodyReadError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    error: Box<dyn std::error::Error + Send + Sync>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl StdError for Error {}

impl From<Infallible> for Error {
    fn from(error: Infallible) -> Error {
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
}

impl fmt::Display for KnownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl StdError for KnownError {}
