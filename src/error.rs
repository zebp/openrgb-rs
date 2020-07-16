use thiserror::Error;

pub type OpenRGBResult<T> = Result<T, OpenRGBError>;

#[derive(Debug, Error)]
pub enum OpenRGBError {
    #[error("io error with tcp stream {0}")]
    IO(#[from] std::io::Error),
}
