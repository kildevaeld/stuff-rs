use thiserror::Error as ThisError;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, ThisError)]
pub enum Error<E> {
    #[error("transport error: {0}")]
    Transport(#[from] E),
    #[error("into request error: {0}")]
    Request(BoxError),
}

impl<E> Error<E> {
    pub fn request<T: std::error::Error + Send + Sync + 'static>(error: T) -> Error<E> {
        Error::Request(Box::new(error))
    }
}
