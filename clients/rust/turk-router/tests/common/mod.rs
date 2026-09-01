//! Shared by the integration tests: where the committed wire artifacts are, and the fixture
//! format. Test-only; nothing here is part of the crate.

#![allow(dead_code)]

pub mod fixture;
pub mod harness;

use std::path::{Path, PathBuf};

pub fn wire_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../wire")
}

pub fn manifest() -> serde_json::Value {
    let path = wire_root().join("wire-manifest.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).expect("the manifest is JSON")
}

/// Every fixture under `wire/fixtures`, sorted by path. Nothing may assume a corpus name, a count
/// or a synthetic address: the delivery that carries a new wire replaces the tree wholesale.
pub fn fixture_paths() -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(&wire_root().join("fixtures"), &mut found);
    found.sort();
    assert!(!found.is_empty(), "wire/fixtures holds no fixture");
    found
}

fn walk(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found);
        } else if path.file_name().is_some_and(|name| name == "accounts.txt") {
            found.push(path);
        }
    }
}
