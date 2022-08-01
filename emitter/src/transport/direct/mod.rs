mod non_send;

pub use self::non_send::*;

#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "sync")]
pub use self::sync::SyncDirect;
