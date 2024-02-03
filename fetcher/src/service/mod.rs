use std::collections::HashMap;

use bytes::Bytes;
use dale::Service;
use mime::Mime;

use crate::{BoxError, ResponseBox};

pub struct Request {
    pub(crate) url: url::Url,
    pub(crate) meta: Option<HashMap<String, String>>,
}

impl Request {
    pub fn new(url: url::Url, meta: impl Into<Option<HashMap<String, String>>>) -> Request {
        Request {
            url,
            meta: meta.into(),
        }
    }

    fn ensure(&mut self) -> &mut HashMap<String, String> {
        if self.meta.is_none() {
            self.meta = Some(HashMap::default());
        }

        self.meta.as_mut().unwrap()
    }

    pub fn set(&mut self, name: impl ToString, value: impl ToString) {
        self.ensure().insert(name.to_string(), value.to_string());
    }
}

impl Request {
    pub fn url(&self) -> &url::Url {
        &self.url
    }
}

pub struct Response {
    pub(crate) meta: HashMap<String, String>,
    pub(crate) mime: Mime,
    pub(crate) len: Option<usize>,
    pub(crate) resp: Option<Box<dyn ResponseBox>>,
}

impl Response {
    pub async fn body(&mut self) -> Result<Bytes, BoxError> {
        self.resp.take().unwrap().body().await
    }
}
