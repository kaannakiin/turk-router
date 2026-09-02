use solana_pubkey::Pubkey;
use turk_router::venues::whirlpool::{self, SupplementalTickArrays, WhirlpoolAccounts};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

const BASE_SLOT_COUNT: usize = 15;

fn key(byte: u8) -> Pubkey {
    Pubkey::new_from_array([byte; 32])
}

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(
        fixture.program_id,
        whirlpool::PROGRAM_ID,
        "{}: not a Whirlpool fixture",
        fixture.pool_b58
    );

    let accounts = WhirlpoolAccounts {
        token_program_a: fixture.slot(0).pubkey,
        token_program_b: fixture.slot(1).pubkey,
        token_authority: fixture.slot(3).pubkey,
        whirlpool: fixture.slot(4).pubkey,
        mint_a: fixture.slot(5).pubkey,
        mint_b: fixture.slot(6).pubkey,
        token_owner_account_a: fixture.slot(7).pubkey,
        token_vault_a: fixture.slot(8).pubkey,
        token_owner_account_b: fixture.slot(9).pubkey,
        token_vault_b: fixture.slot(10).pubkey,
        tick_array_0: fixture.slot(11).pubkey,
        tick_array_1: fixture.slot(12).pubkey,
        tick_array_2: fixture.slot(13).pubkey,
        oracle: fixture.slot(14).pubkey,
    };

    let tail: Vec<Pubkey> = fixture
        .slots
        .get(BASE_SLOT_COUNT..)
        .unwrap_or_default()
        .iter()
        .map(|slot| slot.pubkey)
        .collect();
    let supplemental = match tail.as_slice() {
        [] => SupplementalTickArrays::from([]),
        [a] => SupplementalTickArrays::from([*a]),
        [a, b] => SupplementalTickArrays::from([*a, *b]),
        [a, b, c] => SupplementalTickArrays::from([*a, *b, *c]),
        other => panic!(
            "{}: {} supplemental tick arrays, whirlpool accepts at most 3",
            fixture.pool_b58,
            other.len()
        ),
    };

    whirlpool::resolve(&accounts, supplemental)
}

pub fn reachable_account_counts() -> Vec<u8> {
    let accounts = WhirlpoolAccounts {
        token_program_a: key(1),
        token_program_b: key(2),
        token_authority: key(3),
        whirlpool: key(4),
        mint_a: key(5),
        mint_b: key(6),
        token_owner_account_a: key(7),
        token_vault_a: key(8),
        token_owner_account_b: key(9),
        token_vault_b: key(10),
        tick_array_0: key(11),
        tick_array_1: key(12),
        tick_array_2: key(13),
        oracle: key(14),
    };

    [
        SupplementalTickArrays::from([]),
        SupplementalTickArrays::from([key(20)]),
        SupplementalTickArrays::from([key(20), key(21)]),
        SupplementalTickArrays::from([key(20), key(21), key(22)]),
    ]
    .into_iter()
    .map(|supplemental| whirlpool::resolve(&accounts, supplemental).account_count())
    .collect()
}
