use core::convert::Infallible;

use crate::{IntoOutcome, Outcome, Service};

pub fn any<T: Send>(
) -> impl Service<T, Output = impl IntoOutcome<T, Success = (T, ()), Failure = Infallible>> + Copy {
    |req: T| async move { Outcome::Success((req, ())) }
}

pub fn state<T: Send, S: Send + Clone + 'static>(
    state: S,
) -> impl Service<T, Output = Outcome<(T, (S,)), Infallible, T>> + Clone {
    move |req| {
        let state = state.clone();
        async move { Outcome::Success((req, (state,))) }
    }
}
