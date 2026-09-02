use solana_pubkey::Pubkey;
use turk_router::venues::pump_swap_buy::{self, PumpSwapBuyAccounts};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(fixture.program_id, pump_swap_buy::PROGRAM_ID);
    // The closing pair is always last; whatever sits between slot 22 and it is the optionals,
    // cashback (writable) before pool_v2 (readonly), told apart by the flag when only one is
    // present.
    let optionals = &fixture.slots[23..fixture.slots.len() - 2];
    let (cashback, pool_v2) = match optionals {
        [] => (None, None),
        [one] if one.writable => (Some(one.pubkey), None),
        [one] => (None, Some(one.pubkey)),
        [cashback, pool_v2] => (Some(cashback.pubkey), Some(pool_v2.pubkey)),
        _ => panic!(
            "{}: {} optional tail slots",
            fixture.pool_b58,
            optionals.len()
        ),
    };
    pump_swap_buy::resolve(PumpSwapBuyAccounts {
        pool: fixture.slot(0).pubkey,
        user: fixture.slot(1).pubkey,
        forwarded_before_base_mint: fixture.slot(2).pubkey,
        base_mint: fixture.slot(3).pubkey,
        quote_mint: fixture.slot(4).pubkey,
        base_token_account: fixture.slot(5).pubkey,
        quote_token_account: fixture.slot(6).pubkey,
        base_vault: fixture.slot(7).pubkey,
        quote_vault: fixture.slot(8).pubkey,
        forwarded_before_volume_accumulator: [
            fixture.slot(9).pubkey,
            fixture.slot(10).pubkey,
            fixture.slot(11).pubkey,
            fixture.slot(12).pubkey,
            fixture.slot(13).pubkey,
            fixture.slot(14).pubkey,
            fixture.slot(15).pubkey,
            fixture.slot(16).pubkey,
            fixture.slot(17).pubkey,
            fixture.slot(18).pubkey,
        ],
        user_volume_accumulator: fixture.slot(20).pubkey,
        forwarded_close: [
            fixture.slot(fixture.slots.len() - 2).pubkey,
            fixture.slot(fixture.slots.len() - 1).pubkey,
        ],
        cashback,
        pool_v2,
    })
}

pub fn reachable_account_counts() -> Vec<u8> {
    let key = |byte: u8| Pubkey::new_from_array([byte; 32]);
    let base = || PumpSwapBuyAccounts {
        pool: key(1),
        user: key(2),
        forwarded_before_base_mint: key(3),
        base_mint: key(4),
        quote_mint: key(5),
        base_token_account: key(6),
        quote_token_account: key(7),
        base_vault: key(8),
        quote_vault: key(9),
        forwarded_before_volume_accumulator: [
            key(10),
            key(11),
            key(12),
            key(13),
            key(14),
            key(15),
            key(16),
            key(17),
            key(18),
            key(19),
        ],
        user_volume_accumulator: key(20),
        forwarded_close: [key(21), key(22)],
        pool_v2: None,
        cashback: None,
    };

    vec![
        pump_swap_buy::resolve(base()).account_count(),
        pump_swap_buy::resolve(PumpSwapBuyAccounts {
            pool_v2: Some(key(30)),
            ..base()
        })
        .account_count(),
        pump_swap_buy::resolve(PumpSwapBuyAccounts {
            cashback: Some(key(31)),
            ..base()
        })
        .account_count(),
        pump_swap_buy::resolve(PumpSwapBuyAccounts {
            pool_v2: Some(key(30)),
            cashback: Some(key(31)),
            ..base()
        })
        .account_count(),
    ]
}
