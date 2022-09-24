#[derive(Debug)]
pub enum Error {
    Config(johnfig::Error),
}

impl From<johnfig::Error> for Error {
    fn from(err: johnfig::Error) -> Self {
        Error::Config(err)
    }
}
