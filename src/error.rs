use thiserror::Error;

pub type OpenRGBResult<T> = Result<T, OpenRGBError>;

#[derive(Debug, Error)]
pub enum OpenRGBError {
    #[error("io error with tcp stream {0}")]
    IO(#[from] std::io::Error),
    #[error("invalid command id: {0}")]
    InvalidCommand(u32),
    #[error("invalid element id {0}")]
    InvalidId(usize),
    #[error("incorrect amount of colors expected {0} found {1}")]
    InvalidColorAmount(usize, usize),
    #[error("invalid mode {0}")]
    InvalidMode(String),
}
