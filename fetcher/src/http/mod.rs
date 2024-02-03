use reqwest::Client;

mod request;
mod transport;

mod service;

#[derive(Debug, Default, Clone)]
pub struct Http {
    pub(crate) client: Client,
}
