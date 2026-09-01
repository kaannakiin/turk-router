use solana_pubkey::Pubkey;
use turk_router::venues::meteora_dlmm_swap::{
    resolve as resolve_window, BinArrayTail, MeteoraDlmmSwapAccounts, MAX_BINS, PROGRAM_ID,
};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(fixture.program_id, PROGRAM_ID);

    let accounts = MeteoraDlmmSwapAccounts {
        lb_pair: fixture.slot(0).pubkey,
        bin_array_bitmap_extension: sentinel_to_option(fixture.slot(1).pubkey),
        reserve_x: fixture.slot(2).pubkey,
        reserve_y: fixture.slot(3).pubkey,
        user_token_in: fixture.slot(4).pubkey,
        user_token_out: fixture.slot(5).pubkey,
        mint_x: fixture.slot(6).pubkey,
        mint_y: fixture.slot(7).pubkey,
        oracle: fixture.slot(8).pubkey,
        host_fee_in: sentinel_to_option(fixture.slot(9).pubkey),
        user: fixture.slot(10).pubkey,
    };

    let bins: Vec<Pubkey> = fixture.slots[15..].iter().map(|slot| slot.pubkey).collect();
    let bin_arrays = BinArrayTail::try_from_slice(&bins)
        .unwrap_or_else(|error| panic!("{}: {error}", fixture.pool_b58));

    resolve_window(accounts, bin_arrays)
}

fn sentinel_to_option(pubkey: Pubkey) -> Option<Pubkey> {
    (pubkey != PROGRAM_ID).then_some(pubkey)
}

pub fn reachable_account_counts() -> Vec<u8> {
    fn placeholder(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts_for(bitmap_present: bool, host_fee_present: bool) -> MeteoraDlmmSwapAccounts {
        MeteoraDlmmSwapAccounts {
            lb_pair: placeholder(1),
            bin_array_bitmap_extension: bitmap_present.then(|| placeholder(2)),
            reserve_x: placeholder(3),
            reserve_y: placeholder(4),
            user_token_in: placeholder(5),
            user_token_out: placeholder(6),
            mint_x: placeholder(7),
            mint_y: placeholder(8),
            oracle: placeholder(9),
            host_fee_in: host_fee_present.then(|| placeholder(10)),
            user: placeholder(11),
        }
    }

    let mut counts = Vec::new();
    for bin_count in 1..=MAX_BINS {
        let bins = vec![placeholder(20); bin_count];
        let bin_arrays = BinArrayTail::try_from_slice(&bins).expect("bin_count fits MAX_BINS");
        for bitmap_present in [false, true] {
            for host_fee_present in [false, true] {
                let window =
                    resolve_window(accounts_for(bitmap_present, host_fee_present), bin_arrays);
                counts.push(window.account_count());
            }
        }
    }
    counts
}
