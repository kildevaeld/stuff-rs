use async_oneshot as oneshot;
use std::{future::Future, marker::PhantomData, pin::Pin, sync::Arc};

pub trait Spawner: Send + Sync {
    type Error;
    fn spawn<F: Future + 'static + Send>(
        &self,
        future: F,
    ) -> Pin<Box<dyn Future<Output = Result<F::Output, Self::Error>>>>
    where
        F::Output: Send;
}

#[cfg(feature = "tokio")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokio;
#[cfg(feature = "tokio")]
impl Spawner for Tokio {
    type Error = tokio::task::JoinError;
    fn spawn<F: Future + 'static + Send>(
        &self,
        future: F,
    ) -> Pin<Box<dyn Future<Output = Result<F::Output, Self::Error>>>>
    where
        F::Output: Send,
    {
        Box::pin(tokio::spawn(future))
    }
}

pub struct Context<E, H>
where
    H: Handler<E>,
{
    sx: async_channel::Sender<Message<E, H>>,
}

impl<E, H> Clone for Context<E, H>
where
    H: Handler<E>,
{
    fn clone(&self) -> Self {
        Context {
            sx: self.sx.clone(),
        }
    }
}

impl<E, H> Context<E, H>
where
    H: Handler<E>,
{
    pub fn send(&self, event: E) {
        self.sx
            .try_send(Message {
                context: self.clone(),
                event,
                returns: None,
            })
            .ok();
    }

    pub fn request(&self, event: E) -> impl Future<Output = Result<H::Output, H::Error>> {
        let (sx, rx) = oneshot::oneshot();
        self.sx
            .try_send(Message {
                context: self.clone(),
                event,
                returns: Some(sx),
            })
            .ok();

        async move { rx.await.unwrap() }
    }
}

struct Message<E, H>
where
    H: Handler<E>,
{
    context: Context<E, H>,
    event: E,
    returns: Option<oneshot::Sender<Result<H::Output, H::Error>>>,
}

pub trait Handler<E>: Sized {
    type Output;
    type Error;
    type Future: Future<Output = Result<Self::Output, Self::Error>>;
    fn process(&self, ctx: Context<E, Self>, event: E) -> Self::Future;
}

impl<F, U, O, Err, E> Handler<E> for F
where
    F: Fn(Context<E, F>, E) -> U,
    U: Future<Output = Result<O, Err>>,
{
    type Output = O;
    type Error = Err;
    type Future = U;
    fn process(&self, ctx: Context<E, Self>, event: E) -> Self::Future {
        (self)(ctx, event)
    }
}

pub struct Driver<H, E, S> {
    workers: usize,
    handler: Arc<H>,
    spawner: S,
    _e: PhantomData<E>,
}

impl<H, E, S> Driver<H, E, S>
where
    H: Handler<E>,
{
    pub fn new(spawner: S, handler: H) -> Driver<H, E, S> {
        Driver {
            workers: 0,
            handler: Arc::new(handler),
            spawner,
            _e: PhantomData,
        }
    }

    pub fn workers(&mut self, workers: usize) -> &mut Self {
        self.workers = workers;
        self
    }
}

impl<H, E, S> Driver<H, E, S>
where
    S: Spawner + Clone + 'static,
    E: Send + 'static,
    H: Handler<E> + Send + Sync + 'static,
    H::Future: Send,
    H::Error: Send + Sync,
    H::Output: Send + Sync,
{
    pub async fn run(&self, event: E) -> Result<H::Output, H::Error> {
        let mut ret = self.run_multiple([event]).await;
        ret.pop().unwrap()
    }

    pub async fn run_multiple<I: IntoIterator<Item = E>>(
        &self,
        events: I,
    ) -> Vec<Result<H::Output, H::Error>> {
        let (work_sx, work_rx) = async_channel::bounded::<Message<E, H>>(self.workers.max(1));
        let (msg_sx, msg_rx) = async_channel::unbounded::<Message<E, H>>();

        let handler = self.handler.clone();

        let work_t = self
            .spawner
            .spawn(create_worker(self.spawner.clone(), handler, work_rx));

        let msg_t = self.spawner.spawn(async move {
            while let Ok(msg) = msg_rx.recv().await {
                if work_sx.send(msg).await.is_err() {
                    break;
                }
            }
        });

        let ctx = Context { sx: msg_sx };

        let workers = events.into_iter().map(move |event| ctx.request(event));

        let mut output = Vec::default();

        for worker in workers {
            output.push(worker.await);
        }

        work_t.await.ok();
        msg_t.await.ok();

        output
    }
}

async fn create_worker<S, H, E>(
    spawner: S,
    handler: Arc<H>,
    rx: async_channel::Receiver<Message<E, H>>,
) where
    S: Spawner,
    E: Send + 'static,
    H: Handler<E> + Send + Sync + 'static,
    H::Future: Send,
    H::Error: Send + Sync,
    H::Output: Send + Sync,
{
    while let Ok(next) = rx.recv().await {
        let handler = handler.clone();
        spawner.spawn(async move {
            let ret = handler.process(next.context, next.event).await;
            if let Some(mut returns) = next.returns {
                returns.send(ret).ok();
            }
        });
    }
}
