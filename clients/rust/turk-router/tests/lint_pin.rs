//! The eight-lint deny list at the crate root is policy. Nothing else fails a change that drops
//! or narrows it, so this does.

use std::path::Path;

const DENIED: [&str; 8] = [
    "clippy::arithmetic_side_effects",
    "clippy::cast_possible_truncation",
    "clippy::cast_possible_wrap",
    "clippy::cast_precision_loss",
    "clippy::cast_sign_loss",
    "clippy::expect_used",
    "clippy::float_arithmetic",
    "clippy::unwrap_used",
];

fn sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    collect(&root, &mut found);
    found.sort();
    found
}

fn collect(directory: &Path, found: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(directory)
        .expect("src is readable")
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push((
                path.display().to_string(),
                std::fs::read_to_string(&path).expect("a source file is text"),
            ));
        }
    }
}

#[test]
fn the_crate_root_denies_all_eight_lints() {
    let lib = include_str!("../src/lib.rs");
    let start = lib.find("#![deny(").expect("a deny block");
    let block = &lib[start..lib[start..].find(")]").expect("the block closes") + start];
    for lint in DENIED {
        assert!(
            block.contains(lint),
            "{lint} is missing from the crate-root deny list"
        );
    }
}

#[test]
fn no_source_file_allows_a_clippy_lint() {
    for (path, text) in sources() {
        assert!(
            !text.contains("#[allow(clippy::") && !text.contains("#![allow(clippy::"),
            "{path} allows a clippy lint; the only suppression is #[expect(…, reason)]"
        );
    }
}
