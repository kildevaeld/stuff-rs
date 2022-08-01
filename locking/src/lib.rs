#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "lock"))]
extern crate alloc;

#[cfg(feature = "async")]
mod async_locking;
mod locking;

mod types;

pub use self::{locking::*, types::*};

#[cfg(feature = "async")]
pub use async_locking::*;

#[cfg(feature = "lock")]
pub mod lock;
