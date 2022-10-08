#[cfg(not(feature = "std"))]
use alloc::{rc::Rc, sync::Arc};
#[cfg(feature = "std")]
use std::{rc::Rc, sync::Arc};

use crate::Service;

#[derive(Debug, Clone)]
pub struct SharedService<T> {
    service: Arc<T>,
}

impl<T> SharedService<T> {
    pub fn new(service: T) -> SharedService<T> {
        SharedService {
            service: Arc::new(service),
        }
    }
}

impl<T, R> Service<R> for SharedService<T>
where
    T: Service<R>,
{
    type Output = T::Output;
    type Future = T::Future;

    fn call(&self, req: R) -> Self::Future {
        self.service.call(req)
    }
}

#[derive(Debug, Clone)]
pub struct LocalSharedService<T> {
    service: Rc<T>,
}

impl<T> LocalSharedService<T> {
    pub fn new(service: T) -> LocalSharedService<T> {
        LocalSharedService {
            service: Rc::new(service),
        }
    }
}

impl<T, R> Service<R> for LocalSharedService<T>
where
    T: Service<R>,
{
    type Output = T::Output;
    type Future = T::Future;

    fn call(&self, req: R) -> Self::Future {
        self.service.call(req)
    }
}
