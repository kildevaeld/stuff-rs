use driver::{Driver, Handler, Tokio};
use std::{convert::Infallible, future::Future, pin::Pin};

pub enum Event {
    Greeting,
    Create(String),
    Subject,
}

struct Handle;

impl Handler<Event> for Handle {
    type Input = ();
    type Output = String;

    type Error = Infallible;

    // type Request = ();

    type Future = Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send>>;

    fn process(&self, ctx: driver::Context<Event, Self>, event: Event) -> Self::Future {
        Box::pin(async move {
            match event {
                Event::Greeting => {
                    let subject = ctx.request(Event::Create("Hello".to_string())).await?;
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    Ok(subject)
                }
                Event::Create(greeting) => {
                    let subject = ctx.request(Event::Subject).await?;
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

                    Ok(format!("{}, {}!", greeting, subject))
                }
                Event::Subject => Ok("World".to_string()),
            }
        })
    }
}

#[tokio::main]
async fn main() {
    let mut runner = Driver::new(Tokio, Handle);

    runner.workers(1);

    let ret = runner
        .run_multiple(
            (),
            [
                Event::Greeting,
                Event::Greeting,
                Event::Greeting,
                Event::Greeting,
                Event::Greeting,
                Event::Greeting,
                Event::Create("Hej".to_string()),
                Event::Create("Hejsan".to_string()),
            ],
        )
        .await;

    println!("RET {:?}", ret);
}
