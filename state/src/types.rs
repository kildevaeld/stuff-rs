pub trait IntoInner<T> {
    fn into_inner(self) -> Option<T>;
    fn replace_inner(&self, other: T) -> Option<T>;
}

pub trait Downgrade {
    type Output;
    fn downgrade(&self) -> Self::Output;
}

pub trait Upgrade {
    type Output;
    fn upgrade(self) -> Self::Output;
}
