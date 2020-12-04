#![doc = "commit JSON 映射解析集成测试。"]

use std::fs;

use git_tools::commit::parse_map;

#[test]
fn parse_entries_document() {
    let dir = std::env::temp_dir().join("git-reword-test-map-json");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("map.json");
    fs::write(
        &path,
        r#"{
  "version": 1,
  "entries": [
    { "hash": "abc12345", "message": "✨ Subject line\n\nBody line." },
    { "hash": "abcdef01", "message": "🐛 Fix `thing`" }
  ]
}"#,
    )
    .unwrap();

    let entries = parse_map(&path).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].0, "abc12345");
}

#[test]
fn parse_flat_object() {
    let dir = std::env::temp_dir().join("git-reword-test-map-flat");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("map.json");
    fs::write(&path, r#"{ "abc12345": "✨ Subject\n\nBody." }"#).unwrap();

    let entries = parse_map(&path).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "abc12345");
}
