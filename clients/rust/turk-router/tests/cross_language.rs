//! The cross-language golden corpus, `clients/golden/find_route.json`: what this crate builds for
//! a fixed sweep of inputs, committed so the TypeScript client can be held to the same bytes.
//!
//! Only this file writes the corpus, and only when `TURK_ROUTER_WRITE_GOLDEN` is set; CI never
//! sets it, so a change to what the crate emits turns the first test red until the file is
//! regenerated and the diff reviewed.

mod common;

use std::collections::BTreeSet;

use common::corpus::{
    build_case, error_json, generate, golden_path, read_committed, render, Case, REGENERATE,
};
use common::manifest;
use solana_pubkey::Pubkey;
use turk_router::programs::{ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID};
use turk_router::wire::{CONFIG_SEED, ROUTER_PROGRAM_ID};

#[test]
fn the_committed_corpus_is_what_this_crate_builds() {
    let text = render(&generate());
    let path = golden_path();
    if std::env::var_os("TURK_ROUTER_WRITE_GOLDEN").is_some() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("clients/golden exists");
        }
        std::fs::write(&path, &text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("{}: {error}\nregenerate with: {REGENERATE}", path.display())
    });
    if committed != text {
        let line = committed
            .lines()
            .zip(text.lines())
            .position(|(left, right)| left != right)
            .map_or(
                committed.lines().count().min(text.lines().count()) + 1,
                |index| index + 1,
            );
        panic!(
            "{} differs from what this crate builds, first at line {line}\nregenerate with: {REGENERATE}",
            path.display()
        );
    }
}

/// The reader path: every case's inputs, as committed, are enough to rebuild its output. This is
/// the path the TypeScript client mirrors, so it has to work from the file alone.
#[test]
fn every_committed_case_rebuilds_from_its_own_inputs() {
    let corpus = read_committed();
    assert!(!corpus.cases.is_empty());
    for (id, case) in &corpus.cases {
        assert!(
            case.expected.is_some() != case.error.is_some(),
            "{id}: exactly one of expected and error"
        );
        match build_case(&case.params) {
            Ok(built) => assert_eq!(Some(&built), case.expected.as_ref(), "{id}"),
            Err(error) => assert_eq!(Some(&error_json(&error)), case.error.as_ref(), "{id}"),
        }
    }
}

/// The sweep is the contract: every window length the program accepts, every flag byte, every
/// route-mint count, both base mints, and every error the builder can raise at run time.
#[test]
fn the_sweep_covers_every_window_length_and_every_error() {
    let corpus = generate();
    let manifest = manifest();

    let accepted: BTreeSet<(String, u8)> = manifest["find_route"]["menu_eligible_hop_kinds"]
        .as_array()
        .expect("an array")
        .iter()
        .flat_map(|entry| {
            let name = entry["name"].as_str().expect("a name").to_string();
            entry["window_lens"]
                .as_array()
                .expect("an array")
                .iter()
                .map(move |len| {
                    (
                        name.clone(),
                        u8::try_from(len.as_u64().expect("a length")).expect("u8"),
                    )
                })
        })
        .collect();
    let swept: BTreeSet<(String, u8)> = corpus
        .cases
        .iter()
        .filter(|(id, _)| id.starts_with("window/"))
        .map(|(id, case)| {
            let [window] = case.params.menu.as_slice() else {
                panic!("{id}: a window case names one window");
            };
            let built = common::corpus::window_from_input(window)
                .unwrap_or_else(|error| panic!("{id}: {error}"));
            (format!("{:?}", built.hop_kind()), built.account_count())
        })
        .collect();
    assert_eq!(swept, accepted, "(kind, account_count) pairs");

    let positive: Vec<(&String, &Case)> = corpus
        .cases
        .iter()
        .filter(|(_, case)| case.expected.is_some())
        .collect();
    let flag_bytes: BTreeSet<u8> = positive
        .iter()
        .map(|(_, case)| {
            u8::from(case.params.flags.flashloan)
                | (u8::from(case.params.flags.fail_if_no_profit) << 1)
        })
        .collect();
    assert_eq!(flag_bytes, BTreeSet::from([0, 1, 2, 3]));
    let mint_counts: BTreeSet<usize> = positive
        .iter()
        .map(|(_, case)| case.params.route_mints.len())
        .collect();
    assert_eq!(mint_counts, BTreeSet::from([1, 2, 3, 4]));
    let base_mints: BTreeSet<&str> = positive
        .iter()
        .map(|(_, case)| case.params.base_mint.as_str())
        .collect();
    assert_eq!(base_mints.len(), 2, "both base mints appear");

    let error_kinds: BTreeSet<&str> = corpus
        .cases
        .values()
        .filter_map(|case| case.error.as_ref())
        .map(|error| error["kind"].as_str().expect("an error kind"))
        .collect();
    assert_eq!(
        error_kinds,
        BTreeSet::from([
            "NoRouteMints",
            "TooManyRouteMints",
            "EmptyMenu",
            "TooManyMenuPools",
            "MenuAccountBudgetExceeded",
            "TailLength",
        ])
    );
}

#[test]
fn the_corpus_carries_the_crates_wire_epoch() {
    assert_eq!(generate().wire_epoch, turk_router::WIRE_EPOCH);
    assert_eq!(read_committed().wire_epoch, turk_router::WIRE_EPOCH);
}

/// Derived here independently of the crate: the fee collector's token account for the base mint
/// sits writable at slot 5, the config account readonly at slot 4, in every instruction the
/// corpus records. A client that sends the fee slot readonly fails onchain as
/// `FeeAccountMismatch`; this is where it fails first.
#[test]
fn every_positive_case_sends_the_fee_ata_writable_at_slot_five() {
    let config = Pubkey::find_program_address(&[CONFIG_SEED], &ROUTER_PROGRAM_ID).0;
    let mut checked = 0;
    for (id, case) in &read_committed().cases {
        let Some(expected) = &case.expected else {
            continue;
        };
        let fee_wallet = common::corpus::pubkey(&case.params.fee_wallet);
        let base_mint = common::corpus::base_mint(&case.params.base_mint).mint();
        let fee_ata = Pubkey::find_program_address(
            &[
                fee_wallet.as_ref(),
                TOKEN_PROGRAM_ID.as_ref(),
                base_mint.as_ref(),
            ],
            &ASSOCIATED_TOKEN_PROGRAM_ID,
        )
        .0;
        assert_eq!(expected.accounts[4], format!("{config}:readonly"), "{id}");
        assert_eq!(expected.accounts[5], format!("{fee_ata}:writable"), "{id}");
        checked += 1;
    }
    assert!(checked > 0);
}
