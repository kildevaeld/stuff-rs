pub trait App: Sized {
    type Context;
    type Extension: Extension<Self>;
}

impl App for () {
    type Context = ();
    type Extension = ();
}

pub trait Extension<A> {}

impl<A> Extension<A> for () {}

pub trait IntoExtension<A: App> {
    fn into_extension(self) -> A::Extension;
}
