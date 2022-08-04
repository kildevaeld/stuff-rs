pub trait Downgrade {
    type Output: Upgrade<Output = Self>;
    fn downgrade(&self) -> Self::Output;
}

pub trait Upgrade {
    type Output;
    fn upgrade(&self) -> Option<Self::Output>;
}

pub trait Lockable<'a> {
    type Guard: 'a;
    fn lock(&self) -> Self::Guard;
}

#[cfg(feature = "std")]
impl<'a> Lockable<'a> for std::io::Stdout {
    type Guard = std::io::StdoutLock<'a>;
    fn lock(&self) -> Self::Guard {
        self.lock()
    }
}
