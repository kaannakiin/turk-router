use solana_pubkey::Pubkey;
use turk_router::venues::meteora_dlmm_swap2::{
    resolve as resolve_window, MeteoraDlmmSwap2Accounts, MAX_BIN_ARRAYS, PROGRAM_ID,
};
use turk_router::venues::PubkeyTail;
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

fn optional(pubkey: Pubkey) -> Option<Pubkey> {
    if pubkey == PROGRAM_ID {
        None
    } else {
        Some(pubkey)
    }
}

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(fixture.program_id, PROGRAM_ID);

    let accounts = MeteoraDlmmSwap2Accounts {
        pool: fixture.slot(0).pubkey,
        bin_array_bitmap_extension: optional(fixture.slot(1).pubkey),
        reserve_x: fixture.slot(2).pubkey,
        reserve_y: fixture.slot(3).pubkey,
        user_token_in: fixture.slot(4).pubkey,
        user_token_out: fixture.slot(5).pubkey,
        token_x_mint: fixture.slot(6).pubkey,
        token_y_mint: fixture.slot(7).pubkey,
        oracle: fixture.slot(8).pubkey,
        host_fee_in: optional(fixture.slot(9).pubkey),
        user: fixture.slot(10).pubkey,
        token_x_program: fixture.slot(11).pubkey,
        token_y_program: fixture.slot(12).pubkey,
    };
    // slot(13) memo, slot(14) event_authority, slot(15) program-again are module constants.
    let bin_array_keys: Vec<Pubkey> = fixture.slots[16..].iter().map(|slot| slot.pubkey).collect();
    let bin_arrays = PubkeyTail::<MAX_BIN_ARRAYS>::try_from_slice(&bin_array_keys)
        .expect("fixture bin tail length");

    resolve_window(accounts, bin_arrays)
}

pub fn reachable_account_counts() -> Vec<u8> {
    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    let bin_pool: [Pubkey; MAX_BIN_ARRAYS] = [
        key(101),
        key(102),
        key(103),
        key(104),
        key(105),
        key(106),
        key(107),
        key(108),
    ];

    let mut counts = Vec::new();
    for len in 1..=MAX_BIN_ARRAYS {
        let bin_arrays =
            PubkeyTail::<MAX_BIN_ARRAYS>::try_from_slice(&bin_pool[..len]).expect("len in range");
        for bitmap_extension in [None, Some(key(90))] {
            for host_fee_in in [None, Some(key(91))] {
                let accounts = MeteoraDlmmSwap2Accounts {
                    pool: key(1),
                    bin_array_bitmap_extension: bitmap_extension,
                    reserve_x: key(2),
                    reserve_y: key(3),
                    user_token_in: key(4),
                    user_token_out: key(5),
                    token_x_mint: key(6),
                    token_y_mint: key(7),
                    oracle: key(8),
                    host_fee_in,
                    user: key(9),
                    token_x_program: key(10),
                    token_y_program: key(11),
                };
                counts.push(resolve_window(accounts, bin_arrays).account_count());
            }
        }
    }
    counts
}
