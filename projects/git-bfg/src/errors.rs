use thiserror::Error;

#[derive(Debug, Error)]
pub enum CleanerError {
    #[error(transparent)]
    Discover(#[from] gix::discover::Error),
    #[error(transparent)]
    Object(#[from] gix::object::find::existing::Error),
    #[error(transparent)]
    OdbIter(#[from] gix::odb::store::load_index::Error),
    #[error(transparent)]
    Walkdir(#[from] gix::features::fs::walkdir::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, CleanerError>;

impl CleanerError {
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
