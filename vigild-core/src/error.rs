use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),
    #[error("serialization error: {0}")]
    Serialize(#[from] bincode::Error),
    #[error("journal error: {0}")]
    Journal(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, CoreError>;
