use core::marker::PhantomData;

pub trait Func<T> {
    type Output;
    fn call(&self, input: T) -> Self::Output;
}

impl<T, F, U> Func<T> for F
where
    F: Fn(T) -> U,
{
    type Output = U;

    fn call(&self, input: T) -> Self::Output {
        (self)(input)
    }
}

pub trait Callback<T> {
    type Output;
    fn call(&self, input: &T) -> Self::Output;
}

impl<T, F, U> Callback<T> for F
where
    F: Fn(&T) -> U,
{
    type Output = U;

    fn call(&self, input: &T) -> Self::Output {
        (self)(input)
    }
}

pub trait CallbackExt<T>: Callback<T> {
    fn map<F>(self, func: F) -> CallbackMap<F, Self, T>
    where
        Self: Sized,
        F: Func<Self::Output>,
    {
        CallbackMap {
            callback: self,
            map: func,
            _t: PhantomData,
        }
    }
}

impl<T, C> CallbackExt<T> for C where C: Callback<T> {}

pub struct CallbackMap<F, C, T> {
    callback: C,
    map: F,
    _t: PhantomData<T>,
}

unsafe impl<F, C, T> Sync for CallbackMap<F, C, T>
where
    C: Sync,
    F: Sync,
{
}

unsafe impl<F, C, T> Send for CallbackMap<F, C, T>
where
    C: Send,
    F: Send,
{
}

impl<F, C, T> Callback<T> for CallbackMap<F, C, T>
where
    C: Callback<T>,
    F: Func<C::Output>,
{
    type Output = F::Output;

    fn call(&self, input: &T) -> Self::Output {
        let ret = self.callback.call(input);
        self.map.call(ret)
    }
}

pub trait CallbackMut<T> {
    type Output;
    fn call_mut(&mut self, input: &T) -> Self::Output;
}

pub trait CallbackMutExt<T>: CallbackMut<T> {
    fn map<F>(self, func: F) -> CallbackMutMap<F, Self, T>
    where
        Self: Sized,
        F: Func<Self::Output>,
    {
        CallbackMutMap {
            callback: self,
            map: func,
            _t: PhantomData,
        }
    }
}

impl<T, C> CallbackMutExt<T> for C where C: CallbackMut<T> {}

pub struct CallbackMutMap<F, C, T>
where
    C: CallbackMut<T>,
    F: Func<C::Output>,
{
    callback: C,
    map: F,
    _t: PhantomData<T>,
}

impl<F, C, T> CallbackMut<T> for CallbackMutMap<F, C, T>
where
    C: CallbackMut<T>,
    F: Func<C::Output>,
{
    type Output = F::Output;

    fn call_mut(&mut self, input: &T) -> Self::Output {
        let ret = self.callback.call_mut(input);
        self.map.call(ret)
    }
}

impl<T, F, U> CallbackMut<T> for F
where
    F: FnMut(&T) -> U,
{
    type Output = U;

    fn call_mut(&mut self, input: &T) -> Self::Output {
        (self)(input)
    }
}

impl<'a, E: Clone + 'static> CallbackMut<E> for std::sync::mpsc::Sender<E> {
    type Output = Result<(), std::sync::mpsc::SendError<E>>;
    fn call_mut(&mut self, event: &E) -> Self::Output {
        self.send(event.clone())
    }
}
