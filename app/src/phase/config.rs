use crate::{app::App, Builder};

use super::{Init, Phase};

pub struct Config<A: App> {
    config: johnfig::ConfigBuilder,
    defaults: johnfig::Config,
    initializers: Vec<Box<dyn Fn(&mut Builder<A, Init<A>>)>>,
    extensions: Vec<A::Extension>,
}

impl<A: App> Default for Config<A> {
    fn default() -> Self {
        Config {
            config: johnfig::ConfigBuilder::new(),
            defaults: johnfig::Config::default(),
            initializers: Vec::default(),
            extensions: Vec::default(),
        }
    }
}

impl<A: App> Config<A> {
    pub fn config(&mut self) -> &mut johnfig::ConfigBuilder {
        &mut self.config
    }

    pub fn defaults(&mut self) -> &mut johnfig::Config {
        &mut self.defaults
    }

    pub fn initializers(&mut self) -> &mut Vec<Box<dyn Fn(&mut Builder<A, Init<A>>)>> {
        &mut self.initializers
    }

    pub fn extensions(&mut self) -> &mut Vec<A::Extension> {
        &mut self.extensions
    }

    pub fn build(
        mut self,
        ctx: A::Context,
    ) -> (Init<A>, Vec<Box<dyn Fn(&mut Builder<A, Init<A>>)>>) {
        let config = self.config.build_config().unwrap();

        self.defaults.extend(config);

        let config = self.defaults;

        (Init { config, ctx }, self.initializers)
    }
}

impl<A: App> Phase<A> for Config<A> {}
