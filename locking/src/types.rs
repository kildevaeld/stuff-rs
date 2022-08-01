pub trait Downgrade {
    type Output: Upgrade<Output = Self>;
    fn downgrade(&self) -> Self::Output;
}

pub trait Upgrade {
    type Output;
    fn upgrade(&self) -> Option<Self::Output>;
}
