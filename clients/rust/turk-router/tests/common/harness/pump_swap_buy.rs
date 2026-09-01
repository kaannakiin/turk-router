use solana_pubkey::Pubkey;
use turk_router::venues::pump_swap_buy::{self, PumpSwapBuyAccounts};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(fixture.program_id, pump_swap_buy::PROGRAM_ID);
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
        forwarded_after_fee_program: [fixture.slot(23).pubkey, fixture.slot(24).pubkey],
        pool_v2: fixture.slots.get(25).map(|slot| slot.pubkey),
        cashback: fixture.slots.get(26).map(|slot| slot.pubkey),
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
        forwarded_after_fee_program: [key(21), key(22)],
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
