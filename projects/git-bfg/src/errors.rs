use thiserror::Error;

/// `git-bfg` 操作中的错误类型。
#[derive(Debug, Error)]
pub enum CleanerError {
    /// 发现或打开 git 仓库失败。
    #[error(transparent)]
    Discover(#[from] gix::discover::Error),
    /// 读取对象失败。
    #[error(transparent)]
    Object(#[from] gix::object::find::existing::Error),
    /// 遍历对象数据库失败。
    #[error(transparent)]
    OdbIter(#[from] gix::odb::store::load_index::Error),
    /// 遍历 loose 对象目录失败。
    #[error(transparent)]
    Walkdir(#[from] gix::features::fs::walkdir::Error),
    /// 标准 I/O 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// 通用错误消息。
    #[error("{0}")]
    Message(String),
}

/// `git-bfg` 结果别名。
pub type Result<T> = std::result::Result<T, CleanerError>;

impl CleanerError {
    /// 从字符串构造通用错误。
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
