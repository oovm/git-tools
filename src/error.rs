//! 统一错误类型（基于 `gix-error`）。

use std::borrow::Cow;

use gix_error::{Error as GixError, ValidationError, bstr::BString};

/// 库与 CLI 对外暴露的错误类型。
pub type Error = GixError;

/// 库与 CLI 对外暴露的结果别名。
pub type Result<T = ()> = gix_error::Result<T>;

pub use gix_error::{Exn, Message, OptionExt, ResultExt, ensure, message};

/// 将校验错误转为对外 [`Error`]。
pub fn validation(message: impl Into<Cow<'static, str>>) -> Error {
    Error::from_error(ValidationError::new(message))
}

/// 将带输入上下文的校验错误转为对外 [`Error`]。
pub fn validation_with_input(message: impl Into<Cow<'static, str>>, input: impl Into<BString>) -> Error {
    Error::from_error(ValidationError::new_with_input(message, input))
}
