//! blob 条目：大小、格式与排序/展示。

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use byte_unit::{Byte, UnitType};
use gix::ObjectId;

/// blob 内容格式（文本或二进制）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobFormat {
    /// 含 NUL 字节，视为二进制。
    Binary,
    /// 不含 NUL 字节，视为文本。
    Text,
}

impl BlobFormat {
    /// 根据字节内容推断格式（NUL 字节启发式）。
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.contains(&0) { Self::Binary } else { Self::Text }
    }
}

/// 单个 blob 对象的大小与格式信息。
#[derive(Debug)]
pub struct BlobItem {
    /// 对象 OID。
    pub id: ObjectId,
    /// 字节大小。
    pub size: usize,
    /// 推断的内容格式。
    pub format: BlobFormat,
}

impl Display for BlobItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let size = Byte::from_u64(self.size as u64).get_appropriate_unit(UnitType::Binary);
        write!(f, "{size:>9} | {} | {:?}", self.id, self.format)
    }
}

impl Eq for BlobItem {}

impl PartialEq for BlobItem {
    fn eq(&self, other: &Self) -> bool {
        self.size.eq(&other.size)
    }
}

impl PartialOrd for BlobItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.size.partial_cmp(&other.size)
    }
}

impl Ord for BlobItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size.cmp(&other.size)
    }
}
