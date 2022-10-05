pub type Res<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeJson(e)
    }
}
