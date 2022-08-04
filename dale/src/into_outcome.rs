use core::marker::PhantomData;

use crate::outcome::Outcome;

pub trait IntoOutcome<N> {
    type Success;
    type Failure;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N>;
}

pub trait IntoOutcomeExt<N>: IntoOutcome<N> {
    fn map<F, O>(self, map: F) -> MapOutcome<Self, F, O, N>
    where
        F: FnOnce(Self::Success) -> O,
        Self: Sized,
    {
        MapOutcome {
            parent: self,
            map,
            _o: PhantomData,
            _n: PhantomData,
        }
    }

    fn map_err<F, O>(self, map: F) -> MapErrOutcome<Self, F, O, N>
    where
        F: FnOnce(Self::Failure) -> O,
        Self: Sized,
    {
        MapErrOutcome {
            parent: self,
            map,
            _o: PhantomData,
            _n: PhantomData,
        }
    }
}

impl<I, N> IntoOutcomeExt<N> for I where I: IntoOutcome<N> {}

pub struct MapOutcome<I, F, O, N> {
    parent: I,
    map: F,
    _o: PhantomData<O>,
    _n: PhantomData<N>,
}

impl<I, F, O, N> IntoOutcome<N> for MapOutcome<I, F, O, N>
where
    I: IntoOutcome<N>,
    F: FnOnce(I::Success) -> O,
{
    type Success = O;
    type Failure = I::Failure;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        self.parent.into_outcome().map(self.map)
    }
}

pub struct MapErrOutcome<I, F, O, N> {
    parent: I,
    map: F,
    _o: PhantomData<O>,
    _n: PhantomData<N>,
}

impl<I, F, O, N> IntoOutcome<N> for MapErrOutcome<I, F, O, N>
where
    I: IntoOutcome<N>,
    F: FnOnce(I::Failure) -> O,
{
    type Success = I::Success;
    type Failure = O;

    fn into_outcome(self) -> Outcome<Self::Success, Self::Failure, N> {
        self.parent.into_outcome().map_err(self.map)
    }
}
