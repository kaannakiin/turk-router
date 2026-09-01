//! Every wire number this crate declares has to equal the one `wire/wire-manifest.json` carries.
//!
//! The manifest is generated from the deployed program's own constants, so this is the check that
//! keeps a hand-copied literal from drifting into a client. It needs no network and no secret,
//! which is the point: it runs the same on a fork's pull request as it does on main.

mod common;

use std::collections::BTreeSet;

use common::{harness, manifest};
use serde_json::Value;
use sha2::{Digest, Sha256};
use turk_router::wire::{
    CONFIG_SEED, FIND_ROUTE_DISC, FLAG_FAIL_IF_NO_PROFIT, FLAG_FLASHLOAN, HEADER_LEN,
    MAX_HOOK_GROUP_LEN, MAX_MENU_ACCOUNTS, MAX_MENU_POOLS, MAX_ROUTE_MINTS, MENU_ENTRY_LEN,
    PREFIX_ACCOUNT_METAS, ROUTER_PROGRAM_ID,
};
use turk_router::{BaseMint, HopKind};

fn find_route(manifest: &Value, key: &str) -> Value {
    let value = &manifest["find_route"][key];
    assert!(!value.is_null(), "the manifest has no find_route.{key}");
    value.clone()
}

fn number(manifest: &Value, key: &str) -> u64 {
    find_route(manifest, key)
        .as_u64()
        .unwrap_or_else(|| panic!("find_route.{key} is not a number"))
}

#[test]
fn the_wire_epoch_matches_the_manifest() {
    assert_eq!(
        manifest()["wire_epoch"].as_u64(),
        Some(turk_router::WIRE_EPOCH),
        "the crate and the manifest disagree about which wire they encode for"
    );
}

/// The fields the clients read. A manifest missing one of these is a manifest this repository
/// cannot build against.
#[test]
fn the_manifest_carries_every_field_the_clients_need() {
    let manifest = manifest();
    for key in ["wire_epoch", "program_id", "config_seed", "router_errors"] {
        assert!(!manifest[key].is_null(), "the manifest has no {key}");
    }
    for key in [
        "discriminator",
        "header_len",
        "menu_entry_len",
        "max_menu_pools",
        "max_route_mints",
        "max_menu_accounts",
        "max_hook_group_len",
        "flags",
        "prefix_accounts",
        "prefix_account_metas",
        "base_mints",
        "menu_eligible_hop_kinds",
    ] {
        find_route(&manifest, key);
    }
}

#[test]
fn the_program_and_its_config_seed_match_the_manifest() {
    let manifest = manifest();
    assert_eq!(
        manifest["program_id"].as_str(),
        Some(ROUTER_PROGRAM_ID.to_string().as_str())
    );
    assert_eq!(
        manifest["config_seed"].as_str(),
        Some(std::str::from_utf8(CONFIG_SEED).expect("the seed is text"))
    );
}

#[test]
fn the_discriminator_matches_the_manifest_and_its_own_name() {
    let hex: String = FIND_ROUTE_DISC
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(
        find_route(&manifest(), "discriminator").as_str(),
        Some(hex.as_str())
    );

    let digest = Sha256::digest(b"global:find_route");
    assert_eq!(&digest[..8], &FIND_ROUTE_DISC);
}

#[test]
fn the_widths_and_ceilings_match_the_manifest() {
    let manifest = manifest();
    for (key, declared) in [
        ("header_len", HEADER_LEN),
        ("menu_entry_len", MENU_ENTRY_LEN),
        ("max_menu_pools", MAX_MENU_POOLS),
        ("max_route_mints", MAX_ROUTE_MINTS),
        ("max_menu_accounts", MAX_MENU_ACCOUNTS),
        ("max_hook_group_len", MAX_HOOK_GROUP_LEN),
    ] {
        assert_eq!(number(&manifest, key), declared as u64, "{key}");
    }
}

#[test]
fn the_flag_bits_match_the_manifest() {
    let flags = find_route(&manifest(), "flags");
    assert_eq!(flags["flashloan"].as_u64(), Some(u64::from(FLAG_FLASHLOAN)));
    assert_eq!(
        flags["fail_if_no_profit"].as_u64(),
        Some(u64::from(FLAG_FAIL_IF_NO_PROFIT))
    );
}

#[test]
fn the_prefix_matches_the_manifest_names_and_flags() {
    let manifest = manifest();
    let names: Vec<&str> = find_route(&manifest, "prefix_accounts")
        .as_array()
        .expect("an array")
        .iter()
        .map(|name| name.as_str().expect("a name").to_string())
        .collect::<Vec<_>>()
        .leak()
        .iter()
        .map(String::as_str)
        .collect();
    let declared_names: Vec<&str> = PREFIX_ACCOUNT_METAS
        .iter()
        .map(|(name, _, _)| *name)
        .collect();
    assert_eq!(names, declared_names);

    let metas: Vec<(String, bool, bool)> = find_route(&manifest, "prefix_account_metas")
        .as_array()
        .expect("an array")
        .iter()
        .map(|meta| {
            (
                meta["name"].as_str().expect("a name").to_string(),
                meta["is_signer"].as_bool().expect("a flag"),
                meta["is_writable"].as_bool().expect("a flag"),
            )
        })
        .collect();
    let declared: Vec<(String, bool, bool)> = PREFIX_ACCOUNT_METAS
        .iter()
        .map(|(name, is_signer, is_writable)| ((*name).to_string(), *is_signer, *is_writable))
        .collect();
    assert_eq!(metas, declared);
}

#[test]
fn the_base_mints_match_the_manifest_in_order() {
    let mints: Vec<String> = find_route(&manifest(), "base_mints")
        .as_array()
        .expect("an array")
        .iter()
        .map(|mint| mint.as_str().expect("an address").to_string())
        .collect();
    let declared: Vec<String> = BaseMint::ALL
        .iter()
        .map(|mint| mint.mint().to_string())
        .collect();
    assert_eq!(mints, declared);
}

/// The menu kinds, by discriminant and name, and — for each — the exact set of account counts
/// the venue module can declare, against the lengths the program accepts for a hook-free entry.
/// Equality both ways: a length the module cannot build is liquidity the client cannot route, and
/// a length it builds that the program refuses is an instruction that fails on landing.
#[test]
fn every_menu_kind_builds_exactly_the_window_lengths_the_program_accepts() {
    let entries = find_route(&manifest(), "menu_eligible_hop_kinds");
    let entries = entries.as_array().expect("an array");

    let listed: BTreeSet<u8> = entries
        .iter()
        .map(|entry| u8::try_from(entry["discriminant"].as_u64().expect("a byte")).expect("u8"))
        .collect();
    let declared: BTreeSet<u8> = HopKind::ALL
        .iter()
        .map(|kind| kind.discriminant())
        .collect();
    assert_eq!(
        listed, declared,
        "the manifest and HopKind disagree about the menu set"
    );

    for entry in entries {
        let raw = u8::try_from(entry["discriminant"].as_u64().expect("a byte")).expect("u8");
        let kind = HopKind::try_from(raw).expect("a menu kind");
        assert_eq!(entry["name"].as_str(), Some(format!("{kind:?}").as_str()));

        let accepted: BTreeSet<u8> = entry["window_lens"]
            .as_array()
            .expect("an array")
            .iter()
            .map(|len| u8::try_from(len.as_u64().expect("a length")).expect("u8"))
            .collect();
        let buildable: BTreeSet<u8> = harness::reachable_account_counts(kind)
            .into_iter()
            .collect();
        assert_eq!(
            buildable, accepted,
            "{kind:?}: account counts the module can declare"
        );
    }
}

/// The program can take transfer-hook account groups on two kinds. This crate builds none, and
/// says so; the manifest is the record of which kinds could carry them.
#[test]
fn the_hook_capable_kinds_are_the_two_this_crate_documents_as_unsupported() {
    let entries = find_route(&manifest(), "menu_eligible_hop_kinds");
    let capable: BTreeSet<u8> = entries
        .as_array()
        .expect("an array")
        .iter()
        .filter(|entry| entry["hook_capable"].as_bool() == Some(true))
        .map(|entry| u8::try_from(entry["discriminant"].as_u64().expect("a byte")).expect("u8"))
        .collect();
    let documented: BTreeSet<u8> = [HopKind::Whirlpool, HopKind::MeteoraDlmmSwap2]
        .iter()
        .map(|kind| kind.discriminant())
        .collect();
    assert_eq!(capable, documented);
}

#[test]
fn the_error_numbers_are_dense_from_six_thousand() {
    let manifest = manifest();
    let errors = manifest["router_errors"].as_array().expect("an array");
    assert!(!errors.is_empty());
    for (index, error) in errors.iter().enumerate() {
        assert_eq!(
            error["code"].as_u64(),
            Some(6000 + index as u64),
            "{} is not Custom(6000 + declaration index)",
            error["name"]
        );
    }
}
