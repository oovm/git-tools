#![doc = "commit map 文件解析集成测试。"]

use std::fs;

use git_tools::commit::parse_map_file;

#[test]
fn parse_hash_blocks() {
    let dir = std::env::temp_dir().join("git-reword-test-map");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("map.txt");
    fs::write(&path, "abc12345\n✨ Subject line\n\nBody line.\n\n---\n\nabcdef01\n🐛 Fix `thing`\n").unwrap();

    let entries = parse_map_file(&path).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0, "abc12345");
}
