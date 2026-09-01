//! Every fixture under `wire/fixtures` is read back byte for byte, and every window the venue
//! modules build from a fixture's slots is compared to that fixture slot by slot. The fixtures
//! are the program's own conformance corpus, so this is the check that a module lays accounts out
//! the way the program's adapter reads them.

mod common;

use std::collections::BTreeSet;

use common::fixture::{parse, render, Role};
use common::{fixture_paths, harness};
use turk_router::HopKind;

#[test]
fn every_fixture_round_trips_byte_exactly() {
    for path in fixture_paths() {
        let text = std::fs::read_to_string(&path).expect("a fixture is text");
        let rendered = render(&parse(&text));
        if rendered != text {
            let line = rendered
                .lines()
                .zip(text.lines())
                .position(|(left, right)| left != right)
                .map_or(0, |index| index + 1);
            panic!(
                "{} does not round-trip; first difference at line {line}",
                path.display()
            );
        }
    }
}

#[test]
fn the_bundle_covers_every_menu_kind() {
    let covered: BTreeSet<u8> = fixture_paths()
        .iter()
        .map(|path| parse(&std::fs::read_to_string(path).expect("a fixture is text")).hop_kind)
        .collect();
    for kind in HopKind::ALL {
        assert!(
            covered.contains(&kind.discriminant()),
            "no fixture carries {kind:?}; covered: {covered:?}"
        );
    }
}

/// The `kind:` label names the harness that captured the fixture, not the venue: two kinds share
/// one label. Only `hop_kind:` says which module a fixture belongs to.
#[test]
fn the_kind_label_does_not_identify_the_venue() {
    let fixtures: Vec<_> = fixture_paths()
        .iter()
        .map(|path| parse(&std::fs::read_to_string(path).expect("a fixture is text")))
        .collect();
    let shared = fixtures.iter().any(|left| {
        fixtures
            .iter()
            .any(|right| left.kind == right.kind && left.hop_kind != right.hop_kind)
    });
    assert!(shared, "expected one kind label to span two hop kinds");
}

#[test]
fn every_window_a_module_builds_matches_its_fixture() {
    for path in fixture_paths() {
        let fixture = parse(&std::fs::read_to_string(&path).expect("a fixture is text"));
        assert_eq!(
            fixture.hook_lens,
            [0, 0],
            "{}: this crate builds no transfer-hook groups",
            path.display()
        );

        let window = harness::resolve(&fixture);
        let metas = window.account_metas();
        assert_eq!(
            metas.len(),
            fixture.slots.len() + 1,
            "{}: window length",
            path.display()
        );
        assert_eq!(usize::from(window.account_count()), metas.len());
        assert_eq!(window.hop_kind().discriminant(), fixture.hop_kind);

        let program = &metas[0];
        assert_eq!(
            program.pubkey,
            fixture.program_id,
            "{}: slot 0",
            path.display()
        );
        assert!(
            !program.is_writable && !program.is_signer,
            "{}: slot 0 flags",
            path.display()
        );

        for (index, (meta, slot)) in metas[1..].iter().zip(&fixture.slots).enumerate() {
            let position = index + 1;
            assert_eq!(
                meta.pubkey,
                slot.pubkey,
                "{}: slot {position} address",
                path.display()
            );
            // A payer slot's writable flag is the capturing transaction's, where the same key was
            // also the fee payer; the venue instruction needs the signature, not the write.
            if slot.role != Role::Payer {
                assert_eq!(
                    meta.is_writable,
                    slot.writable,
                    "{}: slot {position} writable",
                    path.display()
                );
            }
            assert_eq!(
                meta.is_signer,
                slot.signer,
                "{}: slot {position} signer",
                path.display()
            );
        }
    }
}
