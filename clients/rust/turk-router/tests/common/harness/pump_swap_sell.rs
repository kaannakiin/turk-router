use solana_pubkey::Pubkey;
use turk_router::venues::pump_swap_sell::{
    resolve as venue_resolve, PumpSwapSellAccounts, PROGRAM_ID,
};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(
        fixture.program_id, PROGRAM_ID,
        "{}: program id",
        fixture.pool_b58
    );

    let tail: Vec<Pubkey> = fixture
        .slots
        .iter()
        .skip(21)
        .map(|slot| slot.pubkey)
        .collect();
    let (cashback, pool_v2, forwarded_close) = match tail.as_slice() {
        [readonly, writable] => (None, None, [*readonly, *writable]),
        [pool_v2, readonly, writable] => (None, Some(*pool_v2), [*readonly, *writable]),
        [first, second, readonly, writable] => {
            (Some([*first, *second]), None, [*readonly, *writable])
        }
        [first, second, pool_v2, readonly, writable] => (
            Some([*first, *second]),
            Some(*pool_v2),
            [*readonly, *writable],
        ),
        other => panic!(
            "{}: unexpected pump swap sell tail length {}",
            fixture.pool_b58,
            other.len()
        ),
    };

    venue_resolve(PumpSwapSellAccounts {
        pool: fixture.slot(0).pubkey,
        user: fixture.slot(1).pubkey,
        forwarded_before_base_mint: fixture.slot(2).pubkey,
        base_mint: fixture.slot(3).pubkey,
        quote_mint: fixture.slot(4).pubkey,
        base_ata: fixture.slot(5).pubkey,
        quote_ata: fixture.slot(6).pubkey,
        base_vault: fixture.slot(7).pubkey,
        quote_vault: fixture.slot(8).pubkey,
        forwarded_before_fee_config: [
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
        cashback,
        pool_v2,
        forwarded_close,
    })
}

pub fn reachable_account_counts() -> Vec<u8> {
    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts(cashback: Option<[Pubkey; 2]>, pool_v2: Option<Pubkey>) -> PumpSwapSellAccounts {
        PumpSwapSellAccounts {
            pool: key(1),
            user: key(2),
            forwarded_before_base_mint: key(3),
            base_mint: key(4),
            quote_mint: key(5),
            base_ata: key(6),
            quote_ata: key(7),
            base_vault: key(8),
            quote_vault: key(9),
            forwarded_before_fee_config: [
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
            cashback,
            pool_v2,
            forwarded_close: [key(20), key(21)],
        }
    }

    let mut counts = Vec::new();
    for cashback in [None, Some([key(31), key(32)])] {
        for pool_v2 in [None, Some(key(30))] {
            counts.push(venue_resolve(accounts(cashback, pool_v2)).account_count());
        }
    }
    counts
}
