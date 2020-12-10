//! blob 清单的人类可读输出（stdout）。

use gix::ObjectId;

use super::blob::BlobItem;

/// 排名表之前的扫描摘要。
pub struct BlobScanSummary {
    /// 已收集的 blob 数量。
    pub blob_count: usize,
    /// 已收集的 tree 数量。
    pub tree_count: usize,
    /// 人类可读的 blob 总大小。
    pub total_size: String,
}

/// 将扫描摘要与前 `show` 个最大 blob 打印到 stdout。
pub fn print_largest_blobs(summary: &BlobScanSummary, ranked: &[BlobItem], corrupt: &[ObjectId], show: usize) {
    println!(
        "Found {} blob(s) and {} tree(s) (total blob size {})",
        summary.blob_count, summary.tree_count, summary.total_size
    );
    println!("Top {} largest blob(s):", show);
    for oid in corrupt {
        println!("{} is missing or corrupt", short_oid(*oid));
    }
    let width = 1 + show.max(1).ilog10() as usize;
    for (index, item) in ranked.iter().take(show).enumerate() {
        println!("{:width$} | {item}", index + 1, width = width);
    }
}

fn short_oid(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
