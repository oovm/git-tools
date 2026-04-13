mod blob_item;

use crate::Result;
use byte_unit::Byte;
use git2::{Blob, ObjectType, Oid, Repository};
use sorted_vec::ReverseSortedVec;
use std::{
    cmp::Ordering,
    env::current_dir,
    ffi::OsString,
    fmt::{Debug, Display, Formatter, Write},
    fs::read_dir,
    path::{Path, PathBuf},
};

pub struct Cleaner {
    repository: Repository,
    trees: Vec<Oid>,
    blobs: Vec<Oid>,
    blob_size: usize,
}

impl Cleaner {
    pub fn new(root: &Path) -> Result<Self> {
        Ok(Self { repository: Repository::open(root)?, trees: vec![], blobs: vec![], blob_size: 0 })
    }
    pub fn clear(&mut self) {
        self.trees.clear();
        self.blobs.clear();
        self.blob_size = 0;
    }
    pub fn collect_info(&mut self) -> Result<()> {
        self.clear();
        let db = self.repository.odb()?;
        db.foreach(|c| {
            let o = match db.read(c.to_owned()) {
                Ok(o) => o,
                Err(_) => {
                    return true;
                }
            };
            match o.kind() {
                ObjectType::Any => {}
                ObjectType::Commit => {}
                ObjectType::Tree => self.trees.push(c.to_owned()),
                ObjectType::Blob => {
                    self.blobs.push(c.to_owned());
                    self.blob_size += o.len()
                }
                ObjectType::Tag => {}
            }
            true
        })?;
        Ok(())
    }
    pub fn largest_objects(&self, show: usize) -> Vec<BlobItem> {
        println!("Finding {} files and {} directories take {}", self.blobs.len(), self.trees.len(), self.all_size());
        println!("Here are {} largest objects:", show);
        let mut sv = ReverseSortedVec::new();
        for i in &self.blobs {
            let blob = match self.repository.find_blob(i.to_owned()) {
                Ok(o) => o,
                Err(_) => {
                    println!("{} had broken", i);
                    continue;
                }
            };
            let item = BlobItem { id: i.to_owned(), format: BlobFormat::from_blob(&blob), size: blob.size() };
            sv.insert(item);
        }
        for (index, item) in sv.iter().take(show).enumerate() {
            println!("{:width$} | {}", index + 1, item, width = 1 + show.ilog10() as usize)
        }
        sv.into_vec()
    }
    pub fn all_size(&self) -> String {
        Byte::from_bytes(self.blob_size as u128).get_appropriate_unit(false).to_string()
    }
}

#[derive(Debug)]
pub struct BlobItem {
    id: Oid,
    size: usize,
    format: BlobFormat,
}

#[derive(Debug)]
pub enum BlobFormat {
    Binary,
    Text,
}

pub fn get_project_root() -> std::io::Result<PathBuf> {
    let path = current_dir()?;
    let mut path_ancestors = path.as_path().ancestors();

    while let Some(p) = path_ancestors.next() {
        let has_cargo = read_dir(p)?.into_iter().any(|p| p.unwrap().file_name() == OsString::from(".git"));
        if has_cargo {
            return Ok(PathBuf::from(p));
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Ran out of places to find Cargo.toml"))
}
