//! Overloadable modification through both owned and mutable references
//! to a type with minimal code duplication.
use http::Response;

use crate::body::Body;

/// Allows use of the implemented type as an argument to Set::set.
///
/// This allows types to be used for ad-hoc overloading of Set::set
/// to perform complex updates to the parameter of Modifier.
pub trait Modifier<F: ?Sized> {
    /// Modify `F` with self.
    fn modify(self, item: &mut F);
}

/// A trait providing the set and set_mut methods for all types.
///
/// Simply implement this for your types and they can be used
/// with modifiers.
pub trait Set {
    /// Modify self using the provided modifier.
    #[inline(always)]
    fn set<M: Modifier<Self>>(mut self, modifier: M) -> Self
    where
        Self: Sized,
    {
        modifier.modify(&mut self);
        self
    }

    /// Modify self through a mutable reference with the provided modifier.
    #[inline(always)]
    fn set_mut<M: Modifier<Self>>(&mut self, modifier: M) -> &mut Self {
        modifier.modify(self);
        self
    }
}

impl<T> Set for Response<T> {}

pub trait With<B> {
    fn with<M: Modifier<Response<B>>>(m: M) -> Response<B>
    where
        B: Body,
    {
        Response::new(B::empty()).set(m)
    }
}

impl<B: Body> With<B> for Response<B> {}
