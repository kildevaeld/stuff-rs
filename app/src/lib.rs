mod app;
mod error;
mod module;
mod phase;

use std::{marker::PhantomData, path::Path};

use app::{App, IntoExtension};
pub use error::Error;
use phase::Phase;

pub struct Builder<A: App, P: Phase<A>> {
    phase: P,
    _a: PhantomData<A>,
}
impl<A: App> Builder<A, phase::Config<A>> {
    pub fn new() -> Builder<A, phase::Config<A>> {
        Builder {
            phase: phase::Config::default(),
            _a: PhantomData,
        }
    }
}

impl<A: App, P: Phase<A>> Builder<A, P> {
    pub fn new_with_phase(phase: P) -> Builder<A, P> {
        Builder {
            phase,
            _a: PhantomData,
        }
    }
}

impl<A: App> Builder<A, phase::Config<A>> {
    pub fn config_path(mut self, path: impl AsRef<Path>) -> Result<Self, Error> {
        self.phase.config().add_search_path(path.as_ref())?;
        Ok(self)
    }

    pub fn init<F: Fn(&mut Builder<A, phase::Init<A>>) + 'static>(mut self, func: F) -> Self {
        self.phase.initializers().push(Box::new(func));
        self
    }

    pub fn register<E>(mut self, extension: E) -> Self
    where
        E: IntoExtension<A>,
    {
        self.phase.extensions().push(extension.into_extension());
        self
    }

    pub fn build(self, ctx: A::Context) -> Builder<A, phase::Ready<A>> {
        let (init, initializers) = self.phase.build(ctx);

        let mut core = Builder::new_with_phase(init);

        for init in initializers {
            init(&mut core);
        }

        todo!()
    }
}
