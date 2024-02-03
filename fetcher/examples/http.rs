use dale::{IntoService, Outcome, Service, ServiceExt};
use fetcher::{service::Request, BoxError, File, Http, ResponseBox, Transport};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), BoxError> {
    let data = Http::default().fetch("http://google.com").await?;

    let service = Http::default()
        .into_service()?
        .or(File::new(".").into_service()?)
        .unify();

    let resp = service
        .call(Request::new("file://./Cargo.toml".parse()?, None))
        .await
        .result()?;

    match resp {
        None => {
            println!("not found")
        }
        Some(mut ret) => {
            let bytes = ret.body().await?;
            println!("{}", String::from_utf8_lossy(&bytes.to_vec()));
        }
    }

    Ok(())
}
