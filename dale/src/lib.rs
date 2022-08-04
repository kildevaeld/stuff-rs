#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

pub mod generic;

mod into_outcome;
mod map;
mod middleware;
mod or;
mod outcome;
mod service;
mod service_ext;

pub mod filters;

#[cfg(feature = "alloc")]
pub mod boxed;

pub use self::{into_outcome::*, middleware::*, outcome::*, service::*, service_ext::*};

pub use self::or::*;

#[cfg(feature = "alloc")]
pub use self::boxed::BoxService;

mod outcome_impl;

pub use either::Either;

mod and;
mod err_into;
mod map_err;

#[cfg(any(feature = "alloc", feature = "std"))]
mod shared;
