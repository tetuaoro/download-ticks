pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[allow(unused)]
    #[error("Data from file is empty")]
    DataFileEmpty,

    #[error("Invalid given datetime")]
    InvalidDatetime,

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
}
