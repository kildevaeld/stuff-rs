use crate::app::App;

pub trait Module<A: App> {
    fn init();
}
