use solana_pubkey::Pubkey;
use turk_router::venues::raydium_amm_v4::{
    resolve as venue_resolve, RaydiumAmmV4Accounts, PROGRAM_ID,
};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(
        fixture.program_id, PROGRAM_ID,
        "{}: program id",
        fixture.pool_b58
    );
    venue_resolve(RaydiumAmmV4Accounts {
        pool: fixture.slot(1).pubkey,
        base_vault: fixture.slot(3).pubkey,
        quote_vault: fixture.slot(4).pubkey,
        user_source: fixture.slot(5).pubkey,
        user_destination: fixture.slot(6).pubkey,
        payer: fixture.slot(7).pubkey,
    })
}

pub fn reachable_account_counts() -> Vec<u8> {
    let key = |byte: u8| Pubkey::new_from_array([byte; 32]);
    vec![venue_resolve(RaydiumAmmV4Accounts {
        pool: key(1),
        base_vault: key(2),
        quote_vault: key(3),
        user_source: key(4),
        user_destination: key(5),
        payer: key(6),
    })
    .account_count()]
}
