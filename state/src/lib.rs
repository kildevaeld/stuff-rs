#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod error;
mod state;
mod types;

#[cfg(feature = "async")]
mod r#async;

pub use self::state::*;

pub use self::{error::*, types::*};

#[cfg(feature = "async")]
pub use self::r#async::*;

#[cfg(feature = "sync")]
pub mod sync;

#[cfg(feature = "sync")]
pub use sync::*;
