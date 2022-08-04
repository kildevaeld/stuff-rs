use core::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::message::Message;

pub trait MessageExt: Message {
    fn join<M: Message, S>(self, msg: M) -> And<Self, M, ()>
    where
        Self: Sized,
    {
        And {
            m1: self,
            m2: msg,
            sep: (),
        }
    }

    fn join_with<M: Message, S: Message>(self, msg: M, sep: S) -> And<Self, M, S>
    where
        Self: Sized,
    {
        And {
            m1: self,
            m2: msg,
            sep,
        }
    }

    fn shared(self) -> SharedMessage<Self>
    where
        Self: Sized,
    {
        SharedMessage {
            message: Arc::new(Mutex::new(self)),
        }
    }

    fn display(self) -> DisplayMessage<Self>
    where
        Self: Sized,
    {
        DisplayMessage(self)
    }
}

impl<M> MessageExt for M where M: Message {}

#[derive(Debug, Clone, PartialEq)]
pub struct And<M1, M2, S> {
    m1: M1,
    m2: M2,
    sep: S,
}

impl<M1, M2, S> Message for And<M1, M2, S>
where
    M1: Message,
    M2: Message,
    S: Message,
{
    fn line_count(&self) -> usize {
        self.m1.line_count() + self.m2.line_count() + self.sep.line_count() - 2
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.m1.message(f)?;
        self.sep.message(f)?;
        self.m2.message(f)?;
        Ok(())
    }

    fn update(&mut self) {
        self.m1.update();
        self.sep.update();
        self.m2.update()
    }
}

pub struct SharedMessage<M> {
    message: Arc<Mutex<M>>,
}

impl<M> SharedMessage<M> {
    pub fn inner<'a>(&'a self) -> MutexGuard<'a, M> {
        self.message.lock().unwrap()
    }
}

impl<M> Clone for SharedMessage<M> {
    fn clone(&self) -> Self {
        SharedMessage {
            message: self.message.clone(),
        }
    }
}

impl<M> Message for SharedMessage<M>
where
    M: Message,
{
    fn line_count(&self) -> usize {
        self.message.lock().unwrap().line_count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.lock().unwrap().message(f)
    }

    fn update(&mut self) {
        self.message.lock().unwrap().update()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayMessage<M>(pub M);

impl<M: Message> fmt::Display for DisplayMessage<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.message(f)
    }
}

impl<M: Message> Message for DisplayMessage<M> {
    fn line_count(&self) -> usize {
        self.0.line_count()
    }

    fn message(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.message(f)
    }

    fn update(&mut self) {
        self.0.update()
    }
}
