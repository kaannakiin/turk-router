use solana_pubkey::Pubkey;
use turk_router::venues::raydium_cpmm::{self, RaydiumCpmmAccounts};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(fixture.program_id, raydium_cpmm::PROGRAM_ID);
    raydium_cpmm::resolve(RaydiumCpmmAccounts {
        user: fixture.slot(0).pubkey,
        amm_config: fixture.slot(2).pubkey,
        pool: fixture.slot(3).pubkey,
        input_token_account: fixture.slot(4).pubkey,
        output_token_account: fixture.slot(5).pubkey,
        input_vault: fixture.slot(6).pubkey,
        output_vault: fixture.slot(7).pubkey,
        input_token_program: fixture.slot(8).pubkey,
        output_token_program: fixture.slot(9).pubkey,
        input_mint: fixture.slot(10).pubkey,
        output_mint: fixture.slot(11).pubkey,
        observation_state: fixture.slot(12).pubkey,
    })
}

pub fn reachable_account_counts() -> Vec<u8> {
    let key = |byte: u8| Pubkey::new_from_array([byte; 32]);
    vec![raydium_cpmm::resolve(RaydiumCpmmAccounts {
        user: key(1),
        amm_config: key(2),
        pool: key(3),
        input_token_account: key(4),
        output_token_account: key(5),
        input_vault: key(6),
        output_vault: key(7),
        input_token_program: key(8),
        output_token_program: key(9),
        input_mint: key(10),
        output_mint: key(11),
        observation_state: key(12),
    })
    .account_count()]
}
