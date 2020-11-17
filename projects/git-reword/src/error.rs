use thiserror::Error;

/// `git-reword` 操作中的错误类型。
#[derive(Debug, Error)]
pub enum RewordError {
    /// 发现或打开 git 仓库失败。
    #[error(transparent)]
    Discover(#[from] gix::discover::Error),
    /// 解析 revision 规范失败。
    #[error(transparent)]
    Revision(#[from] gix::revision::spec::parse::single::Error),
    /// 查找引用失败。
    #[error(transparent)]
    Reference(#[from] gix::reference::find::Error),
    /// 读取 HEAD 失败。
    #[error(transparent)]
    Head(#[from] gix::reference::find::existing::Error),
    /// 查找对象失败。
    #[error(transparent)]
    Object(#[from] gix::object::find::Error),
    /// 查找对象失败（必须存在）。
    #[error(transparent)]
    ObjectExisting(#[from] gix::object::find::existing::Error),
    /// 解码 commit 对象失败。
    #[error(transparent)]
    Decode(#[from] gix::objs::decode::Error),
    /// 配置 revision walk 失败。
    #[error(transparent)]
    Walk(#[from] gix::revision::walk::Error),
    /// revision walk 迭代失败。
    #[error(transparent)]
    WalkIter(#[from] gix::revision::walk::iter::Error),
    /// 写入对象失败。
    #[error(transparent)]
    Write(#[from] gix::object::write::Error),
    /// 更新引用失败。
    #[error(transparent)]
    RefEdit(#[from] gix::reference::edit::Error),
    /// 标准 I/O 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// 通用错误消息。
    #[error("{0}")]
    Message(String),
}

/// `git-reword` 结果别名。
pub type Result<T> = std::result::Result<T, RewordError>;

impl RewordError {
    /// 从字符串构造通用错误。
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
