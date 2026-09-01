//! Every wire number this crate declares has to equal the one `wire/wire-manifest.json` carries.
//!
//! The manifest is generated from the deployed program's own constants, so this is the check that
//! keeps a hand-copied literal from drifting into a client. It needs no network and no secret,
//! which is the point: it runs the same on a fork's pull request as it does on main.

use std::path::Path;

fn manifest() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../wire/wire-manifest.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_str(&text).expect("the manifest is JSON")
}

#[test]
fn the_wire_epoch_matches_the_manifest() {
    assert_eq!(
        manifest()["wire_epoch"].as_u64(),
        Some(turk_router::WIRE_EPOCH),
        "the crate and the manifest disagree about which wire they encode for"
    );
}

/// The fields the clients will read as they are written. A manifest missing one of these is a
/// manifest this repository cannot build against, and finding that out here beats finding it out
/// halfway through a venue module.
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
        "prefix_accounts",
        "menu_eligible_hop_kinds",
    ] {
        assert!(
            !manifest["find_route"][key].is_null(),
            "the manifest has no find_route.{key}"
        );
    }
}
