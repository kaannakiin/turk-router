use solana_pubkey::Pubkey;
use turk_router::venues::meteora_damm_v2::{
    resolve as build_window, DammV2Form, MeteoraDammV2Accounts, PROGRAM_ID,
};
use turk_router::VenueWindow;

use crate::common::fixture::Fixture;

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    assert_eq!(fixture.program_id, PROGRAM_ID);

    let referral_slot = fixture.slot(11).pubkey;
    let accounts = MeteoraDammV2Accounts {
        pool: fixture.slot(1).pubkey,
        input_token_account: fixture.slot(2).pubkey,
        output_token_account: fixture.slot(3).pubkey,
        token_a_vault: fixture.slot(4).pubkey,
        token_b_vault: fixture.slot(5).pubkey,
        token_a_mint: fixture.slot(6).pubkey,
        token_b_mint: fixture.slot(7).pubkey,
        payer: fixture.slot(8).pubkey,
        token_a_program: fixture.slot(9).pubkey,
        token_b_program: fixture.slot(10).pubkey,
        referral_token_account: (referral_slot != PROGRAM_ID).then_some(referral_slot),
    };

    let form = match fixture.slots.len() {
        14 => DammV2Form::Base,
        15 => DammV2Form::RateLimited,
        other => panic!(
            "{}: unexpected meteora damm v2 slot count {other}",
            fixture.pool_b58
        ),
    };

    build_window(accounts, form)
}

pub fn reachable_account_counts() -> Vec<u8> {
    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts(referral_token_account: Option<Pubkey>) -> MeteoraDammV2Accounts {
        MeteoraDammV2Accounts {
            pool: key(1),
            input_token_account: key(2),
            output_token_account: key(3),
            token_a_vault: key(4),
            token_b_vault: key(5),
            token_a_mint: key(6),
            token_b_mint: key(7),
            payer: key(8),
            token_a_program: key(9),
            token_b_program: key(10),
            referral_token_account,
        }
    }

    let mut counts = Vec::new();
    for form in [DammV2Form::Base, DammV2Form::RateLimited] {
        for referral_token_account in [None, Some(key(11))] {
            counts.push(build_window(accounts(referral_token_account), form).account_count());
        }
    }
    counts
}
