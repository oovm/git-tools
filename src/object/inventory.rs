//! 遍历对象库、统计 blob 并输出大小排名。

use std::{collections::HashSet, path::Path};

use byte_unit::{Byte, UnitType};
use gix::{ObjectId, Repository, objs::Kind};

use crate::error::{Result, ResultExt, message};

use super::blob::{BlobFormat, BlobItem};

/// 扫描 git 对象库并统计 blob / tree。
pub struct Cleaner {
    repo: Repository,
    trees: Vec<ObjectId>,
    blobs: Vec<ObjectId>,
    blob_size: u64,
}

impl Cleaner {
    /// 打开指定路径下的 git 仓库。
    pub fn new(root: &Path) -> Result<Self> {
        let repo = gix::discover(root).or_raise(|| message!("discover git repository"))?;
        Ok(Self { repo, trees: vec![], blobs: vec![], blob_size: 0 })
    }

    /// 清空已收集的统计信息。
    pub fn clear(&mut self) {
        self.trees.clear();
        self.blobs.clear();
        self.blob_size = 0;
    }

    /// 遍历对象库，收集 blob 与 tree 的 OID 及 blob 总大小。
    pub fn collect_info(&mut self) -> Result<()> {
        self.clear();
        let mut seen = HashSet::new();
        for oid in self.repo.objects.store_ref().iter().or_raise(|| message!("iterate object database"))? {
            let oid = oid.or_raise(|| message!("read object id from odb iterator"))?;
            if !seen.insert(oid) {
                continue;
            }
            let header = match self.repo.find_header(oid) {
                Ok(header) => header,
                Err(_) => continue,
            };
            match header.kind() {
                Kind::Tree => self.trees.push(oid),
                Kind::Blob => {
                    self.blob_size += header.size() as u64;
                    self.blobs.push(oid);
                }
                Kind::Commit | Kind::Tag => {}
            }
        }
        Ok(())
    }

    /// 打印并返回按大小降序排列的前 `show` 个 blob。
    pub fn largest_objects(&self, show: usize) -> Vec<BlobItem> {
        println!("Found {} blob(s) and {} tree(s) (total blob size {})", self.blobs.len(), self.trees.len(), self.all_size());
        println!("Top {} largest blob(s):", show);
        let mut ranked = Vec::with_capacity(self.blobs.len());
        for oid in &self.blobs {
            let blob = match self.repo.find_blob(*oid) {
                Ok(blob) => blob,
                Err(_) => {
                    println!("{} is missing or corrupt", short_oid(*oid));
                    continue;
                }
            };
            ranked.push(BlobItem { id: *oid, format: BlobFormat::from_bytes(&blob.data), size: blob.data.len() });
        }
        ranked.sort_by_key(|item| std::cmp::Reverse(item.size));
        let width = 1 + show.max(1).ilog10() as usize;
        for (index, item) in ranked.iter().take(show).enumerate() {
            println!("{:width$} | {item}", index + 1, width = width);
        }
        ranked.into_iter().take(show).collect()
    }

    /// 返回已收集 blob 的总大小（人类可读字符串）。
    pub fn all_size(&self) -> String {
        Byte::from_u64(self.blob_size).get_appropriate_unit(UnitType::Binary).to_string()
    }
}

/// 将 OID 格式化为 8 位十六进制前缀。
fn short_oid(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
