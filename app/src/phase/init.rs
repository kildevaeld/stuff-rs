use crate::app::App;

use super::Phase;

pub struct Init<A: App> {
    pub(crate) config: johnfig::Config,
    pub(crate) ctx: A::Context,
}

impl<A: App> Phase<A> for Init<A> {}
