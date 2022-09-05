use runner::{Handler, Runner, Tokio};
use std::{convert::Infallible, future::Future, pin::Pin};

pub enum Event {
    Greeting,
    Create(String),
    Subject,
}

struct Handle;

impl Handler<Event> for Handle {
    type Output = String;

    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;

    fn process(&self, ctx: runner::Context<Event, Self>, event: Event) -> Self::Future {
        Box::pin(async move {
            match event {
                Event::Greeting => {
                    let subject = ctx.request(Event::Create("Hello".to_string())).await?;
                    Ok(subject)
                }
                Event::Create(greeting) => {
                    let subject = ctx.request(Event::Subject).await?;
                    Ok(format!("{}, {}!", greeting, subject))
                }
                Event::Subject => Ok("World".to_string()),
            }
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut runner = Runner::new(Tokio, Handle);

    runner.workers(4);

    let ret = runner
        .run_multiple([Event::Greeting, Event::Create("Hej".to_string())])
        .await;
}
