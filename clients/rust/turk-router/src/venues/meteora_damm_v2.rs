//! Meteora DAMM v2 (`cp-amm`), the `swap2` instruction.
//!
//! # Window
//!
//! `account_count` is 15 for [`DammV2Form::Base`], or 16 for [`DammV2Form::RateLimited`], which
//! appends the instructions sysvar the rate-limiter fee mode requires.
//!
//! # Slots
//!
//! 0. the venue program
//! 1. `pool_authority`
//! 2. `pool` (writable)
//! 3. `input_token_account` (writable)
//! 4. `output_token_account` (writable)
//! 5. `token_a_vault` (writable)
//! 6. `token_b_vault` (writable)
//! 7. `token_a_mint`
//! 8. `token_b_mint`
//! 9. `payer` (signer)
//! 10. `token_a_program`
//! 11. `token_b_program`
//! 12. `referral_token_account`
//! 13. `event_authority`
//! 14. the venue program again
//! 15. the instructions sysvar, present only in the `RateLimited` form
//!
//! # Token programs
//!
//! Token Program and Token-2022, independently per mint: `token_a_program` and
//! `token_b_program` need not match.

use solana_pubkey::Pubkey;

use crate::programs::INSTRUCTIONS_SYSVAR_ID;
use crate::venues::{readonly, signer, writable, VenueWindow};
use crate::HopKind;

/// `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG` — the Meteora DAMM v2 (`cp-amm`) program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    9, 45, 33, 53, 101, 122, 21, 156, 43, 135, 212, 182, 106, 112, 219, 142, 151, 82, 56, 159, 247,
    106, 175, 32, 108, 237, 6, 58, 56, 249, 90, 237,
]);

/// `HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC` — the pool authority PDA every pool shares.
pub const POOL_AUTHORITY: Pubkey = Pubkey::new_from_array([
    242, 204, 213, 53, 172, 165, 241, 115, 106, 200, 34, 221, 7, 115, 228, 217, 47, 189, 138, 89,
    178, 148, 3, 80, 2, 149, 169, 1, 28, 115, 169, 229,
]);

/// `3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet` — the event-CPI authority PDA every pool
/// shares.
pub const EVENT_AUTHORITY: Pubkey = Pubkey::new_from_array([
    42, 118, 231, 179, 68, 100, 10, 28, 252, 89, 76, 139, 202, 208, 160, 145, 1, 28, 172, 125, 209,
    86, 191, 131, 168, 51, 251, 34, 8, 235, 119, 173,
]);

const BASE_ACCOUNT_COUNT: u8 = 15;
const RATE_LIMITED_ACCOUNT_COUNT: u8 = 16;

/// Whether the window carries the instructions sysvar a rate-limiter pool requires.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DammV2Form {
    /// The 15-account window: no rate limiter.
    Base,
    /// The 16-account window: the instructions sysvar follows the venue program.
    RateLimited,
}

/// The caller-supplied accounts for one Meteora DAMM v2 `swap2` hop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeteoraDammV2Accounts {
    /// The pool.
    pub pool: Pubkey,
    /// The user's token account for the side being sold.
    pub input_token_account: Pubkey,
    /// The user's token account for the side being bought.
    pub output_token_account: Pubkey,
    /// The pool's vault for `token_a_mint`.
    pub token_a_vault: Pubkey,
    /// The pool's vault for `token_b_mint`.
    pub token_b_vault: Pubkey,
    /// The pool's canonical token-A mint.
    pub token_a_mint: Pubkey,
    /// The pool's canonical token-B mint.
    pub token_b_mint: Pubkey,
    /// The transaction signer whose token account is debited.
    pub payer: Pubkey,
    /// The token program owning `token_a_mint`.
    pub token_a_program: Pubkey,
    /// The token program owning `token_b_mint`.
    pub token_b_program: Pubkey,
    /// The pool's referral fee token account, when the pool has one.
    pub referral_token_account: Option<Pubkey>,
}

/// Builds the window for one Meteora DAMM v2 hop.
#[must_use]
pub fn resolve(accounts: MeteoraDammV2Accounts, form: DammV2Form) -> VenueWindow {
    let referral_token_account = accounts.referral_token_account.unwrap_or(PROGRAM_ID);
    let mut metas = vec![
        readonly(PROGRAM_ID),
        readonly(POOL_AUTHORITY),
        writable(accounts.pool),
        writable(accounts.input_token_account),
        writable(accounts.output_token_account),
        writable(accounts.token_a_vault),
        writable(accounts.token_b_vault),
        readonly(accounts.token_a_mint),
        readonly(accounts.token_b_mint),
        signer(accounts.payer),
        readonly(accounts.token_a_program),
        readonly(accounts.token_b_program),
        readonly(referral_token_account),
        readonly(EVENT_AUTHORITY),
        readonly(PROGRAM_ID),
    ];
    let account_count = match form {
        DammV2Form::Base => BASE_ACCOUNT_COUNT,
        DammV2Form::RateLimited => {
            metas.push(readonly(INSTRUCTIONS_SYSVAR_ID));
            RATE_LIMITED_ACCOUNT_COUNT
        }
    };
    VenueWindow::new(HopKind::MeteoraDammV2, account_count, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn the_addresses_are_the_documented_ones() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG"
        );
        assert_eq!(
            POOL_AUTHORITY.to_string(),
            "HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC"
        );
        assert_eq!(
            EVENT_AUTHORITY.to_string(),
            "3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet"
        );
    }

    #[test]
    fn account_count_reflects_the_form() {
        assert_eq!(
            resolve(accounts(Some(key(11))), DammV2Form::Base).account_count(),
            15
        );
        assert_eq!(
            resolve(accounts(None), DammV2Form::Base).account_count(),
            15
        );
        assert_eq!(
            resolve(accounts(Some(key(11))), DammV2Form::RateLimited).account_count(),
            16
        );
        assert_eq!(
            resolve(accounts(None), DammV2Form::RateLimited).account_count(),
            16
        );
    }

    #[test]
    fn an_absent_referral_account_falls_back_to_the_program_id() {
        let window = resolve(accounts(None), DammV2Form::Base);
        assert_eq!(window.account_metas()[12].pubkey, PROGRAM_ID);
    }

    #[test]
    fn the_rate_limited_form_appends_the_instructions_sysvar() {
        let window = resolve(accounts(None), DammV2Form::RateLimited);
        let metas = window.account_metas();
        assert_eq!(metas.len(), 16);
        assert_eq!(metas[15].pubkey, INSTRUCTIONS_SYSVAR_ID);
        assert!(!metas[15].is_writable);
        assert!(!metas[15].is_signer);
    }

    #[test]
    fn slot_flags_match_the_swap2_account_list() {
        let window = resolve(accounts(Some(key(11))), DammV2Form::Base);
        let metas = window.account_metas();
        let writable_slots = [2, 3, 4, 5, 6];
        for (index, meta) in metas.iter().enumerate() {
            assert_eq!(
                meta.is_writable,
                writable_slots.contains(&index),
                "slot {index}"
            );
            assert_eq!(meta.is_signer, index == 9, "slot {index}");
        }
    }
}
