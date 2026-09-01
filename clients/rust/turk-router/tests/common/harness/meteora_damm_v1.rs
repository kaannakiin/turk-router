use solana_pubkey::Pubkey;
use turk_router::venues::meteora_damm_v1::{
    resolve as venue_resolve, MeteoraDammV1Accounts, PROGRAM_ID,
};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(
        fixture.program_id, PROGRAM_ID,
        "{}: program id",
        fixture.pool_b58
    );
    venue_resolve(MeteoraDammV1Accounts {
        pool: fixture.slot(0).pubkey,
        user_source: fixture.slot(1).pubkey,
        user_dest: fixture.slot(2).pubkey,
        a_vault: fixture.slot(3).pubkey,
        b_vault: fixture.slot(4).pubkey,
        a_token_vault: fixture.slot(5).pubkey,
        b_token_vault: fixture.slot(6).pubkey,
        a_vault_lp_mint: fixture.slot(7).pubkey,
        b_vault_lp_mint: fixture.slot(8).pubkey,
        a_vault_lp: fixture.slot(9).pubkey,
        b_vault_lp: fixture.slot(10).pubkey,
        protocol_token_fee: fixture.slot(11).pubkey,
        payer: fixture.slot(12).pubkey,
    })
}

pub fn reachable_account_counts() -> Vec<u8> {
    let key = |byte: u8| Pubkey::new_from_array([byte; 32]);
    vec![venue_resolve(MeteoraDammV1Accounts {
        pool: key(1),
        user_source: key(2),
        user_dest: key(3),
        a_vault: key(4),
        b_vault: key(5),
        a_token_vault: key(6),
        b_token_vault: key(7),
        a_vault_lp_mint: key(8),
        b_vault_lp_mint: key(9),
        a_vault_lp: key(10),
        b_vault_lp: key(11),
        protocol_token_fee: key(12),
        payer: key(13),
    })
    .account_count()]
}
