use thiserror::Error;

#[derive(Debug, Error)]
pub enum RewordError {
    #[error(transparent)]
    Discover(#[from] gix::discover::Error),
    #[error(transparent)]
    Revision(#[from] gix::revision::spec::parse::single::Error),
    #[error(transparent)]
    Reference(#[from] gix::reference::find::Error),
    #[error(transparent)]
    Head(#[from] gix::reference::find::existing::Error),
    #[error(transparent)]
    Object(#[from] gix::object::find::Error),
    #[error(transparent)]
    ObjectExisting(#[from] gix::object::find::existing::Error),
    #[error(transparent)]
    Decode(#[from] gix::objs::decode::Error),
    #[error(transparent)]
    Walk(#[from] gix::revision::walk::Error),
    #[error(transparent)]
    WalkIter(#[from] gix::revision::walk::iter::Error),
    #[error(transparent)]
    Write(#[from] gix::object::write::Error),
    #[error(transparent)]
    RefEdit(#[from] gix::reference::edit::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, RewordError>;

impl RewordError {
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
