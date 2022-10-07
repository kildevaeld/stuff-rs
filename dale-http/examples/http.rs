use dale_http::prelude::*;
use dale_http::{filters, reply, Result};
use hyper::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let service = filters::get()
        .and(filters::path("/"))
        .map(|| "Hello, World!")
        .or(filters::get()
            .and(filters::path("/test"))
            .map(|| reply::html("<h1>Hello, World!</h1>")))
        .or(filters::method()
            .and_then(|(req, (method,))| async move {
                //
                Result::Ok("And then this")
            })
            .shared());

    let service = dale_http::hyper::MakeTaskHyperService::new(service);

    Server::bind(&addr).serve(service).await?;

    Ok(())
}
