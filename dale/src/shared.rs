#[cfg(not(feature = "std"))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;

use crate::Service;

#[derive(Debug)]
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

impl<T> Clone for SharedService<T> {
    fn clone(&self) -> Self {
        SharedService {
            service: self.service.clone(),
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
