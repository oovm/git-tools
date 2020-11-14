use std::{
    cmp::Ordering,
    fmt::{Display, Formatter},
};

use byte_unit::{Byte, UnitType};
use gix::ObjectId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobFormat {
    Binary,
    Text,
}

impl BlobFormat {
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.contains(&0) { Self::Binary } else { Self::Text }
    }
}

#[derive(Debug)]
pub struct BlobItem {
    pub id: ObjectId,
    pub size: usize,
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
