#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Outcome<S, E, N> {
    Success(S),
    Failure(E),
    Next(N),
}

impl<S, E, N> Outcome<S, E, N> {
    #[inline]
    pub fn map<F, O>(self, map: F) -> Outcome<O, E, N>
    where
        F: FnOnce(S) -> O,
    {
        match self {
            Outcome::Success(success) => Outcome::Success(map(success)),
            Outcome::Failure(err) => Outcome::Failure(err),
            Outcome::Next(n) => Outcome::Next(n),
        }
    }

    #[inline]
    pub fn map_into<T>(self) -> Outcome<T, E, N>
    where
        S: Into<T>,
    {
        match self {
            Outcome::Success(success) => Outcome::Success(success.into()),
            Outcome::Failure(err) => Outcome::Failure(err),
            Outcome::Next(n) => Outcome::Next(n),
        }
    }

    #[inline]
    pub fn map_err<F, O>(self, map: F) -> Outcome<S, O, N>
    where
        F: FnOnce(E) -> O,
    {
        match self {
            Outcome::Success(success) => Outcome::Success(success),
            Outcome::Failure(err) => Outcome::Failure(map(err)),
            Outcome::Next(n) => Outcome::Next(n),
        }
    }

    #[inline]
    pub fn err_into<T>(self) -> Outcome<S, T, N>
    where
        E: Into<T>,
    {
        match self {
            Outcome::Success(success) => Outcome::Success(success),
            Outcome::Failure(err) => Outcome::Failure(err.into()),
            Outcome::Next(n) => Outcome::Next(n),
        }
    }

    pub fn map_next<F, M>(self, func: F) -> Outcome<S, E, M>
    where
        F: FnOnce(N) -> M,
    {
        match self {
            Outcome::Success(success) => Outcome::Success(success),
            Outcome::Failure(err) => Outcome::Failure(err),
            Outcome::Next(n) => Outcome::Next(func(n)),
        }
    }
}
