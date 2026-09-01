//! PumpSwap, buying the base token.
//!
//! # Window
//!
//! `account_count` is 26 with neither optional account present, 27 with exactly one of
//! `pool_v2`/`cashback` present, or 28 with both present.
//!
//! # Slots
//!
//! 0. Program (readonly)
//! 1. `pool` (writable)
//! 2. `user` (writable, signer)
//! 3. forwarded (role withheld) (readonly)
//! 4. `base_mint` (readonly)
//! 5. `quote_mint` (readonly)
//! 6. `base_token_account` (writable)
//! 7. `quote_token_account` (writable)
//! 8. `base_vault` (writable)
//! 9. `quote_vault` (writable)
//! 10. forwarded (role withheld) (readonly)
//! 11. forwarded (role withheld) (writable)
//! 12. forwarded (role withheld) (readonly)
//! 13. forwarded (role withheld) (readonly)
//! 14. forwarded (role withheld) (readonly)
//! 15. forwarded (role withheld) (readonly)
//! 16. forwarded (role withheld) (readonly)
//! 17. forwarded (role withheld) (readonly)
//! 18. forwarded (role withheld) (writable)
//! 19. forwarded (role withheld) (readonly)
//! 20. `GLOBAL_VOLUME_ACCUMULATOR` (readonly)
//! 21. `user_volume_accumulator` (writable)
//! 22. `FEE_CONFIG` (readonly)
//! 23. `FEE_PROGRAM` (readonly)
//! 24. forwarded (role withheld) (readonly)
//! 25. forwarded (role withheld) (writable)
//! 26. `pool_v2` (readonly), present only when `pool_v2` is `Some`
//! 27. `cashback` (writable), present only when `cashback` is `Some`
//!
//! `base_token_account` and `quote_token_account` are the user's own associated token accounts:
//! this module's caller validates them, and the program checks their owner against `user`.
//!
//! # Variable tail
//!
//! `pool_v2` (slot 26) and `cashback` (slot 27) are each present or absent independently, so
//! `account_count` is 26 plus one for each that is `Some`. When only `cashback` is present it
//! occupies slot 26, not 27: the two are appended in order, and neither ever leaves a gap.
//!
//! # Token programs
//!
//! PumpSwap accepts SPL Token and Token Extensions mints on either leg. The token program
//! accounts are forwarded positions this module does not read; the caller supplies whichever
//! program each mint uses.

use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

use crate::venues::{readonly, writable, VenueWindow};
use crate::HopKind;

/// `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` — the PumpSwap AMM program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    12, 20, 222, 252, 130, 94, 198, 118, 148, 37, 8, 24, 187, 101, 64, 101, 244, 41, 141, 49, 86,
    213, 113, 180, 212, 248, 9, 12, 24, 233, 168, 99,
]);

/// `C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw` — the global volume accumulator every buy
/// names at slot 20.
pub const GLOBAL_VOLUME_ACCUMULATOR: Pubkey = Pubkey::new_from_array([
    163, 215, 187, 18, 126, 88, 173, 193, 44, 166, 143, 131, 67, 126, 194, 225, 195, 249, 130, 13,
    233, 62, 88, 249, 23, 138, 41, 24, 221, 170, 247, 180,
]);

/// `5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx` — the fee configuration account every buy
/// names at slot 22.
pub const FEE_CONFIG: Pubkey = Pubkey::new_from_array([
    65, 36, 110, 204, 125, 120, 254, 129, 228, 23, 115, 164, 105, 101, 65, 153, 55, 146, 58, 7,
    100, 71, 151, 223, 111, 62, 181, 20, 66, 96, 16, 203,
]);

/// `pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ` — the external fee program every buy names at
/// slot 23.
pub const FEE_PROGRAM: Pubkey = Pubkey::new_from_array([
    12, 53, 255, 169, 5, 90, 142, 86, 141, 168, 247, 188, 7, 86, 21, 39, 76, 241, 201, 44, 164, 31,
    64, 0, 156, 81, 106, 164, 20, 194, 124, 112,
]);

const BASE_ACCOUNT_COUNT: u8 = 26;

/// Ten consecutive forwarded slots (window 10..=19), readonly or writable per position, in
/// order.
const FORWARDED_MID_WRITABLE: [bool; 10] = [
    false, true, false, false, false, false, false, false, true, false,
];

/// The accounts a PumpSwap buy (`buy_exact_quote_in`) takes, named in window order. Positions the
/// adapter forwards without reading are grouped as fixed arrays with their role withheld.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PumpSwapBuyAccounts {
    /// The pool state account.
    pub pool: Pubkey,
    /// The signer whose token accounts the swap moves through.
    pub user: Pubkey,
    /// Window slot 3, forwarded without validation.
    pub forwarded_before_base_mint: Pubkey,
    /// The mint the swap receives.
    pub base_mint: Pubkey,
    /// The mint the swap spends.
    pub quote_mint: Pubkey,
    /// `user`'s associated token account for `base_mint`.
    pub base_token_account: Pubkey,
    /// `user`'s associated token account for `quote_mint`.
    pub quote_token_account: Pubkey,
    /// The pool's vault for `base_mint`.
    pub base_vault: Pubkey,
    /// The pool's vault for `quote_mint`.
    pub quote_vault: Pubkey,
    /// Window slots 10..=19, forwarded without validation.
    pub forwarded_before_volume_accumulator: [Pubkey; 10],
    /// `user`'s volume accumulator PDA.
    pub user_volume_accumulator: Pubkey,
    /// Window slots 24..=25, forwarded without validation.
    pub forwarded_after_fee_program: [Pubkey; 2],
    /// The pool-v2 sibling account, present only for pools that name one.
    pub pool_v2: Option<Pubkey>,
    /// The cashback ledger account, present only for cashback pools.
    pub cashback: Option<Pubkey>,
}

/// Builds the window for a PumpSwap buy.
#[must_use]
pub fn resolve(accounts: PumpSwapBuyAccounts) -> VenueWindow {
    let mut metas = vec![
        readonly(PROGRAM_ID),
        writable(accounts.pool),
        AccountMeta::new(accounts.user, true),
        readonly(accounts.forwarded_before_base_mint),
        readonly(accounts.base_mint),
        readonly(accounts.quote_mint),
        writable(accounts.base_token_account),
        writable(accounts.quote_token_account),
        writable(accounts.base_vault),
        writable(accounts.quote_vault),
    ];
    for (key, is_writable) in accounts
        .forwarded_before_volume_accumulator
        .iter()
        .zip(FORWARDED_MID_WRITABLE)
    {
        metas.push(if is_writable {
            writable(*key)
        } else {
            readonly(*key)
        });
    }
    metas.push(readonly(GLOBAL_VOLUME_ACCUMULATOR));
    metas.push(writable(accounts.user_volume_accumulator));
    metas.push(readonly(FEE_CONFIG));
    metas.push(readonly(FEE_PROGRAM));
    metas.push(readonly(accounts.forwarded_after_fee_program[0]));
    metas.push(writable(accounts.forwarded_after_fee_program[1]));

    let mut account_count = BASE_ACCOUNT_COUNT;
    if let Some(pool_v2) = accounts.pool_v2 {
        metas.push(readonly(pool_v2));
        account_count = account_count.checked_add(1).unwrap_or(account_count);
    }
    if let Some(cashback) = accounts.cashback {
        metas.push(writable(cashback));
        account_count = account_count.checked_add(1).unwrap_or(account_count);
    }

    VenueWindow::new(HopKind::PumpSwapBuy, account_count, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts() -> PumpSwapBuyAccounts {
        PumpSwapBuyAccounts {
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
        }
    }

    #[test]
    fn the_program_id_is_the_documented_one() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA"
        );
    }

    #[test]
    fn the_global_volume_accumulator_is_the_documented_one() {
        assert_eq!(
            GLOBAL_VOLUME_ACCUMULATOR.to_string(),
            "C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw"
        );
    }

    #[test]
    fn the_fee_config_is_the_documented_one() {
        assert_eq!(
            FEE_CONFIG.to_string(),
            "5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx"
        );
    }

    #[test]
    fn the_fee_program_is_the_documented_one() {
        assert_eq!(
            FEE_PROGRAM.to_string(),
            "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ"
        );
    }

    #[test]
    fn the_base_window_declares_twenty_six_accounts() {
        let window = resolve(accounts());
        assert_eq!(window.account_count(), 26);
        assert_eq!(window.account_metas().len(), 26);
        assert_eq!(window.hop_kind(), HopKind::PumpSwapBuy);
    }

    #[test]
    fn a_pool_v2_sibling_grows_the_window_by_one() {
        let window = resolve(PumpSwapBuyAccounts {
            pool_v2: Some(key(30)),
            ..accounts()
        });
        assert_eq!(window.account_count(), 27);
        assert_eq!(window.account_metas().len(), 27);
    }

    #[test]
    fn a_cashback_ledger_grows_the_window_by_one() {
        let window = resolve(PumpSwapBuyAccounts {
            cashback: Some(key(31)),
            ..accounts()
        });
        assert_eq!(window.account_count(), 27);
        assert_eq!(window.account_metas().len(), 27);
    }

    #[test]
    fn both_optionals_grow_the_window_by_two() {
        let window = resolve(PumpSwapBuyAccounts {
            pool_v2: Some(key(30)),
            cashback: Some(key(31)),
            ..accounts()
        });
        assert_eq!(window.account_count(), 28);
        assert_eq!(window.account_metas().len(), 28);
    }

    #[test]
    fn the_leading_slot_is_the_program_readonly() {
        let window = resolve(accounts());
        let program = &window.account_metas()[0];
        assert_eq!(program.pubkey, PROGRAM_ID);
        assert!(!program.is_writable && !program.is_signer);
    }

    #[test]
    fn the_user_slot_is_writable_and_signed() {
        let window = resolve(accounts());
        let user = &window.account_metas()[2];
        assert_eq!(user.pubkey, key(2));
        assert!(user.is_writable && user.is_signer);
    }
}
