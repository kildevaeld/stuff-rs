pub trait MapFunc<A> {
    type Output;
    fn call(&self, args: A) -> Self::Output;
}

impl<F, U, A> MapFunc<A> for F
where
    F: Fn(A) -> U,
{
    type Output = U;
    fn call(&self, args: A) -> Self::Output {
        (self)(args)
    }
}
