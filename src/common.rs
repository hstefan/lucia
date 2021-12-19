use thiserror::Error;

#[derive(Error, Debug)]
pub enum LuciaError {
    #[error("failure to resolve mdns query")]
    MulticastDnsFailure(#[from] mdns::Error),
    #[error("http request failure")]
    HttpFailure(#[from] reqwest::Error),
    #[error("invalid address")]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("polling time exceeded")]
    PollingTimeout,
    #[error("unable to load/store json data")]
    JsonErron(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, LuciaError>;
