use std::iter::Skip;

use dale::{BoxService, Outcome};
use value::Value;

pub enum Error {}

pub trait Enabler<C> {
    fn enabled(&self, ctx: &C) -> bool;
}

impl<C> Enabler<C> for Box<dyn Enabler<C>> {
    fn enabled(&self, ctx: &C) -> bool {
        (&**self).enabled(ctx)
    }
}

pub trait Skipper<C> {
    fn skip(&self, ctx: &C) -> bool;
}

impl<C> Skipper<C> for Box<dyn Skipper<C>> {
    fn skip(&self, ctx: &C) -> bool {
        (&**self).skip(ctx)
    }
}

pub enum Response<C> {
    Value(Value),
    List(Listr<C>),
}

pub enum Finale {
    Value(Value),
    Multi(Vec<Value>),
    Skip,
}

pub struct WorkDesc<C> {
    title: String,
    task: BoxService<'static, C, Response<C>, Error>,
    enabler: Option<Box<dyn Enabler<C>>>,
    skip: Option<Box<dyn Skipper<C>>>,
}

pub struct Listr<C> {
    tasks: Vec<WorkDesc<C>>,
    ctx: C,
    concurrently: bool,
}

impl<C: Clone> Listr<C> {
    async fn run_task(&self, task: &WorkDesc<C>) -> Result<Finale, Error> {
        if let Some(enabler) = &task.enabler {
            if !enabler.enabled(&self.ctx) {
                return Ok(Finale::Skip);
            }
        }

        if let Some(skipper) = &task.skip {
            if !skipper.skip(&self.ctx) {
                return Ok(Finale::Skip);
            }
        }

        let ret = match task.task.call(self.ctx.clone()).await {
            Outcome::Failure(err) => {
                todo!("failure")
            }
            Outcome::Next(_) => {
                todo!("next")
            }
            Outcome::Success(ret) => ret,
        };

        todo!()
    }
    async fn concurrent_run(&self) {}

    async fn serial_run(&self) {
        for work in &self.tasks {
            self.run_task(work).await;
        }
    }

    pub async fn run(&self) {
        if self.concurrently {
            self.concurrent_run().await;
        } else {
            self.serial_run().await;
        }
    }
}
