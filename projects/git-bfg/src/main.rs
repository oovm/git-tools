#![doc = include_str!("../Readme.md")]

mod cleaner;
mod errors;

pub use errors::{CleanerError, Result};

pub use self::cleaner::{get_project_root, Cleaner};

fn main() -> Result<()> {
    let root = get_project_root()?;
    let mut cleaner = Cleaner::new(&root)?;
    cleaner.collect_info()?;
    cleaner.largest_objects(100);

    // for i in cleaner.trees {
    //     let r = cleaner.repository.find_tree(i)?;
    //     for e in r.iter() {
    //         println!("{:?}", e.name())
    //     }
    // }
    Ok(())
}
