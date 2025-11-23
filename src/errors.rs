pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Check the first/last kline data.")]
    MissingData,

    #[error("Unable to read you input file. Make sure it is json kline valid data.")]
    InvalidFile,

    #[error("Invalid given datetime.")]
    InvalidDatetime,

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),
}
