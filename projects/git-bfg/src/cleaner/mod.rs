mod blob_item;

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use byte_unit::{Byte, UnitType};
use gix::{ObjectId, Repository, objs::Kind};

use crate::Result;

pub use blob_item::{BlobFormat, BlobItem};

pub struct Cleaner {
    repo: Repository,
    trees: Vec<ObjectId>,
    blobs: Vec<ObjectId>,
    blob_size: u64,
}

impl Cleaner {
    pub fn new(root: &Path) -> Result<Self> {
        Ok(Self { repo: gix::discover(root)?, trees: vec![], blobs: vec![], blob_size: 0 })
    }

    pub fn clear(&mut self) {
        self.trees.clear();
        self.blobs.clear();
        self.blob_size = 0;
    }

    pub fn collect_info(&mut self) -> Result<()> {
        self.clear();
        let mut seen = HashSet::new();
        for oid in self.repo.objects.store_ref().iter()? {
            let oid = oid?;
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
        }        ranked.into_iter().take(show).collect()
    }

    pub fn all_size(&self) -> String {
        Byte::from_u64(self.blob_size).get_appropriate_unit(UnitType::Binary).to_string()
    }
}

pub fn find_git_root(start: PathBuf) -> Result<PathBuf> {
    let mut path = start;
    loop {
        if path.join(".git").exists() {
            return Ok(path);
        }
        if !path.pop() {
            return Err(crate::CleanerError::msg("no `.git` directory found in ancestors"));
        }
    }
}

fn short_oid(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
