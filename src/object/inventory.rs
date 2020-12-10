//! 遍历对象库、统计 blob 并输出大小排名。

use std::{collections::HashSet, path::Path};

use byte_unit::{Byte, UnitType};
use gix::{ObjectId, Repository, objs::Kind};
use tracing::instrument;

use crate::error::{Result, ResultExt, message};

use super::{
    blob::{BlobFormat, BlobItem},
    report::BlobScanSummary,
};

/// 对象库扫描后的 blob 排名结果。
pub struct BlobRanking {
    /// 扫描摘要计数。
    pub summary: BlobScanSummary,
    /// 按大小降序排列的 blob。
    pub ranked: Vec<BlobItem>,
    /// 无法读取的 blob OID。
    pub corrupt: Vec<ObjectId>,
}

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
    #[instrument(skip(self))]
    pub fn collect_info(&mut self) -> Result<()> {
        self.clear();
        tracing::debug!("scanning object database");
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
                    self.blob_size += header.size();
                    self.blobs.push(oid);
                }
                Kind::Commit | Kind::Tag => {}
            }
        }
        tracing::info!(
            blobs = self.blobs.len(),
            trees = self.trees.len(),
            total_blob_bytes = self.blob_size,
            "object database scan complete"
        );
        Ok(())
    }

    /// 返回按大小降序排列的 blob 排名（不写入 stdout）。
    #[instrument(skip(self))]
    pub fn rank_largest_blobs(&self) -> BlobRanking {
        let mut ranked = Vec::with_capacity(self.blobs.len());
        let mut corrupt = Vec::new();
        for oid in &self.blobs {
            let blob = match self.repo.find_blob(*oid) {
                Ok(blob) => blob,
                Err(_) => {
                    tracing::warn!(oid = %short_oid(*oid), "blob object missing or corrupt");
                    corrupt.push(*oid);
                    continue;
                }
            };
            ranked.push(BlobItem { id: *oid, format: BlobFormat::from_bytes(&blob.data), size: blob.data.len() });
        }
        ranked.sort_by_key(|item| std::cmp::Reverse(item.size));
        BlobRanking {
            summary: BlobScanSummary {
                blob_count: self.blobs.len(),
                tree_count: self.trees.len(),
                total_size: self.all_size(),
            },
            ranked,
            corrupt,
        }
    }

    /// 返回已收集 blob 的总大小（人类可读字符串）。
    pub fn all_size(&self) -> String {
        Byte::from_u64(self.blob_size).get_appropriate_unit(UnitType::Binary).to_string()
    }
}

fn short_oid(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
