use solana_pubkey::Pubkey;
use turk_router::venues::raydium_clmm::{self, ClmmTail, RaydiumClmmAccounts};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

fn key(byte: u8) -> Pubkey {
    Pubkey::new_from_array([byte; 32])
}

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(fixture.program_id, raydium_clmm::PROGRAM_ID);
    let accounts = RaydiumClmmAccounts {
        payer: fixture.slot(0).pubkey,
        amm_config: fixture.slot(1).pubkey,
        pool: fixture.slot(2).pubkey,
        input_token_account: fixture.slot(3).pubkey,
        output_token_account: fixture.slot(4).pubkey,
        input_vault: fixture.slot(5).pubkey,
        output_vault: fixture.slot(6).pubkey,
        observation_state: fixture.slot(7).pubkey,
        input_mint: fixture.slot(11).pubkey,
        output_mint: fixture.slot(12).pubkey,
    };

    let mut tail_keys = fixture.slots.iter().skip(13).map(|slot| slot.pubkey);
    let first = tail_keys.next().expect("fixture carries a tail account");
    let mut tail = ClmmTail::new(first);
    for tail_key in tail_keys {
        tail.push(tail_key)
            .expect("fixture tail fits the module's bound");
    }

    raydium_clmm::resolve(accounts, tail)
}

fn placeholder_accounts() -> RaydiumClmmAccounts {
    RaydiumClmmAccounts {
        payer: key(1),
        amm_config: key(2),
        pool: key(3),
        input_token_account: key(4),
        output_token_account: key(5),
        input_vault: key(6),
        output_vault: key(7),
        observation_state: key(8),
        input_mint: key(9),
        output_mint: key(10),
    }
}

pub fn reachable_account_counts() -> Vec<u8> {
    (1..=7u8)
        .map(|len| {
            let mut tail = ClmmTail::new(key(100));
            for extra in 1..len {
                tail.push(key(100 + extra)).expect("within the tail bound");
            }
            raydium_clmm::resolve(placeholder_accounts(), tail).account_count()
        })
        .collect()
}
