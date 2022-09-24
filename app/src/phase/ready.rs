use std::marker::PhantomData;

use crate::app::App;

use super::Phase;

pub struct Ready<A: App> {
    _a: PhantomData<A>,
}

impl<A: App> Phase<A> for Ready<A> {}
