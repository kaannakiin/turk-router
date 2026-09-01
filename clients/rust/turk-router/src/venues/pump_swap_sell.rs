//! PumpSwap, the `sell` instruction (base mint → quote mint).
//!
//! # Window
//!
//! `account_count` is `24..=27`: a 24-account base window, plus 1 when `pool_v2` is present and 2
//! when `cashback` is present.
//!
//! # Slots
//!
//! 0. [`PROGRAM_ID`] (readonly).
//! 1. `pool` (writable).
//! 2. `user` (writable, signer) — the wallet authorizing the swap.
//! 3. `forwarded_before_base_mint` (readonly) — role withheld.
//! 4. `base_mint` (readonly).
//! 5. `quote_mint` (readonly).
//! 6. `base_ata` (writable) — the caller's token account for `base_mint`, the sell's input.
//! 7. `quote_ata` (writable) — the caller's token account for `quote_mint`, the sell's output.
//! 8. `base_vault` (writable) — the pool's vault for `base_mint`.
//! 9. `quote_vault` (writable) — the pool's vault for `quote_mint`.
//! 10. `forwarded_before_fee_config[0]` (readonly) — role withheld.
//! 11. `forwarded_before_fee_config[1]` (writable) — role withheld.
//! 12. `forwarded_before_fee_config[2]` (readonly) — role withheld.
//! 13. `forwarded_before_fee_config[3]` (readonly) — role withheld.
//! 14. `forwarded_before_fee_config[4]` (readonly) — role withheld.
//! 15. `forwarded_before_fee_config[5]` (readonly) — role withheld.
//! 16. `forwarded_before_fee_config[6]` (readonly) — role withheld.
//! 17. `forwarded_before_fee_config[7]` (readonly) — role withheld.
//! 18. `forwarded_before_fee_config[8]` (writable) — role withheld.
//! 19. `forwarded_before_fee_config[9]` (readonly) — role withheld.
//! 20. [`FEE_CONFIG`] (readonly).
//! 21. [`FEE_PROGRAM`] (readonly).
//!
//! Positions `22` through `26` are the variable tail — see "Variable tail" below.
//!
//! Slots 3 and 10 through 19 are never read by the adapter that accepts this window; their role
//! names are withheld here until cross-checked against the venue's published IDL. The caller
//! supplies each in the order its own pool source provides.
//!
//! # Variable tail
//!
//! After slot 21, in order: the two `cashback` accounts when present (both writable, role
//! withheld), the one `pool_v2` account when present (readonly, role withheld), then
//! `forwarded_close`'s two accounts, which close every window regardless of the other two
//! (readonly, then writable; role withheld).
//!
//! # Token programs
//!
//! Window positions `[12]` and `[13]` — `forwarded_before_fee_config[2]` and `[3]` — carry the
//! token programs for `base_mint` and `quote_mint` respectively. Either may be the Token program
//! or the Token Extensions program, independently of the other.

use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

use super::{readonly, writable, VenueWindow};
use crate::HopKind;

/// `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` — the PumpSwap AMM program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    12, 20, 222, 252, 130, 94, 198, 118, 148, 37, 8, 24, 187, 101, 64, 101, 244, 41, 141, 49, 86,
    213, 113, 180, 212, 248, 9, 12, 24, 233, 168, 99,
]);

/// `5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx` — the fee-config PDA every pool shares.
pub const FEE_CONFIG: Pubkey = Pubkey::new_from_array([
    65, 36, 110, 204, 125, 120, 254, 129, 228, 23, 115, 164, 105, 101, 65, 153, 55, 146, 58, 7,
    100, 71, 151, 223, 111, 62, 181, 20, 66, 96, 16, 203,
]);

/// `pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ` — the external fee program every pool shares.
pub const FEE_PROGRAM: Pubkey = Pubkey::new_from_array([
    12, 53, 255, 169, 5, 90, 142, 86, 141, 168, 247, 188, 7, 86, 21, 39, 76, 241, 201, 44, 164, 31,
    64, 0, 156, 81, 106, 164, 20, 194, 124, 112,
]);

const BASE_ACCOUNT_COUNT: u8 = 24;
const POOL_V2_LEN: u8 = 1;
const CASHBACK_LEN: u8 = 2;

/// Whether window position `[10 + index]` is writable, for `index` `0..=9`.
const FORWARDED_MID_WRITABLE: [bool; 10] = [
    false, true, false, false, false, false, false, false, true, false,
];

/// The caller-supplied accounts for one PumpSwap `sell` hop, named in window order. Fields named
/// `forwarded_*` carry accounts this crate never reads; see the module's "Slots" section for the
/// exact window position and flags of each element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PumpSwapSellAccounts {
    /// The pool this hop trades against.
    pub pool: Pubkey,
    /// The wallet authorizing the swap. Signs.
    pub user: Pubkey,
    /// Window position `[3]`.
    pub forwarded_before_base_mint: Pubkey,
    /// The pool's base mint.
    pub base_mint: Pubkey,
    /// The pool's quote mint.
    pub quote_mint: Pubkey,
    /// The caller's token account for `base_mint` — the sell's input.
    pub base_ata: Pubkey,
    /// The caller's token account for `quote_mint` — the sell's output.
    pub quote_ata: Pubkey,
    /// The pool's vault for `base_mint`.
    pub base_vault: Pubkey,
    /// The pool's vault for `quote_mint`.
    pub quote_vault: Pubkey,
    /// Window positions `[10]` through `[19]`, in order. Positions `[12]`/`[13]` (elements `2`
    /// and `3`) carry the token programs for `base_mint`/`quote_mint`.
    pub forwarded_before_fee_config: [Pubkey; 10],
    /// The pool's cashback volume-ledger pair, present only on a cashback pool. Both accounts are
    /// writable.
    pub cashback: Option<[Pubkey; 2]>,
    /// The pool's `pool-v2` sidecar account, present only when the pool has a creator set.
    /// Readonly.
    pub pool_v2: Option<Pubkey>,
    /// The two accounts that close every window after `cashback` and `pool_v2`, whether or not
    /// either is present: readonly, then writable.
    pub forwarded_close: [Pubkey; 2],
}

#[expect(
    clippy::expect_used,
    reason = "BASE_ACCOUNT_COUNT (24) plus CASHBACK_LEN (2) plus POOL_V2_LEN (1) is 27 at most, \
              well inside u8::MAX"
)]
fn account_count(has_cashback: bool, has_pool_v2: bool) -> u8 {
    let mut count = BASE_ACCOUNT_COUNT;
    if has_cashback {
        count = count.checked_add(CASHBACK_LEN).expect("fits u8");
    }
    if has_pool_v2 {
        count = count.checked_add(POOL_V2_LEN).expect("fits u8");
    }
    count
}

/// Builds the window for one PumpSwap `sell` hop.
///
/// Infallible: `cashback` and `pool_v2` are already-bounded `Option`s, so every input maps to one
/// of the four accepted account counts.
#[must_use]
pub fn resolve(accounts: PumpSwapSellAccounts) -> VenueWindow {
    let PumpSwapSellAccounts {
        pool,
        user,
        forwarded_before_base_mint,
        base_mint,
        quote_mint,
        base_ata,
        quote_ata,
        base_vault,
        quote_vault,
        forwarded_before_fee_config,
        cashback,
        pool_v2,
        forwarded_close,
    } = accounts;

    let count = account_count(cashback.is_some(), pool_v2.is_some());
    let mut metas = Vec::with_capacity(usize::from(count));
    metas.push(readonly(PROGRAM_ID));
    metas.push(writable(pool));
    metas.push(AccountMeta::new(user, true));
    metas.push(readonly(forwarded_before_base_mint));
    metas.push(readonly(base_mint));
    metas.push(readonly(quote_mint));
    metas.push(writable(base_ata));
    metas.push(writable(quote_ata));
    metas.push(writable(base_vault));
    metas.push(writable(quote_vault));
    for (key, mid_writable) in forwarded_before_fee_config
        .into_iter()
        .zip(FORWARDED_MID_WRITABLE)
    {
        metas.push(if mid_writable {
            writable(key)
        } else {
            readonly(key)
        });
    }
    metas.push(readonly(FEE_CONFIG));
    metas.push(readonly(FEE_PROGRAM));
    if let Some([first, second]) = cashback {
        metas.push(writable(first));
        metas.push(writable(second));
    }
    if let Some(key) = pool_v2 {
        metas.push(readonly(key));
    }
    let [close_readonly, close_writable] = forwarded_close;
    metas.push(readonly(close_readonly));
    metas.push(writable(close_writable));

    VenueWindow::new(HopKind::PumpSwapSell, count, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn sample_accounts(
        cashback: Option<[Pubkey; 2]>,
        pool_v2: Option<Pubkey>,
    ) -> PumpSwapSellAccounts {
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

    #[test]
    fn the_addresses_are_the_documented_ones() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA"
        );
        assert_eq!(
            FEE_CONFIG.to_string(),
            "5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx"
        );
        assert_eq!(
            FEE_PROGRAM.to_string(),
            "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ"
        );
    }

    #[test]
    fn account_count_reflects_the_optional_tail() {
        assert_eq!(resolve(sample_accounts(None, None)).account_count(), 24);
        assert_eq!(
            resolve(sample_accounts(None, Some(key(30)))).account_count(),
            25
        );
        assert_eq!(
            resolve(sample_accounts(Some([key(31), key(32)]), None)).account_count(),
            26
        );
        assert_eq!(
            resolve(sample_accounts(Some([key(31), key(32)]), Some(key(30)))).account_count(),
            27
        );
    }

    #[test]
    fn every_construction_declares_a_count_matching_its_metas_len() {
        for cashback in [None, Some([key(31), key(32)])] {
            for pool_v2 in [None, Some(key(30))] {
                let window = resolve(sample_accounts(cashback, pool_v2));
                assert_eq!(
                    usize::from(window.account_count()),
                    window.account_metas().len()
                );
                assert_eq!(window.hop_kind(), HopKind::PumpSwapSell);
            }
        }
    }

    #[test]
    fn slot_zero_is_the_program_readonly() {
        let window = resolve(sample_accounts(None, None));
        let program = window.account_metas().first().unwrap();
        assert_eq!(program.pubkey, PROGRAM_ID);
        assert!(!program.is_writable && !program.is_signer);
    }

    #[test]
    fn slot_two_is_the_only_signer_and_carries_user() {
        let window = resolve(sample_accounts(None, None));
        let metas = window.account_metas();
        for (index, meta) in metas.iter().enumerate() {
            assert_eq!(meta.is_signer, index == 2, "slot {index}");
        }
        assert_eq!(metas.get(2).unwrap().pubkey, key(2));
    }

    #[test]
    fn the_fixed_prefix_carries_the_documented_writable_flags() {
        let window = resolve(sample_accounts(None, None));
        let metas = window.account_metas();
        let writable_slots = [1, 2, 6, 7, 8, 9, 11, 18];
        for (index, meta) in metas.iter().enumerate().take(22) {
            assert_eq!(
                meta.is_writable,
                writable_slots.contains(&index),
                "slot {index}"
            );
        }
    }

    #[test]
    fn fee_config_and_fee_program_sit_at_twenty_and_twenty_one() {
        let window = resolve(sample_accounts(None, None));
        let metas = window.account_metas();
        assert_eq!(metas.get(20).unwrap().pubkey, FEE_CONFIG);
        assert_eq!(metas.get(21).unwrap().pubkey, FEE_PROGRAM);
    }

    #[test]
    fn cashback_precedes_pool_v2_which_precedes_the_closing_pair() {
        let window = resolve(sample_accounts(Some([key(31), key(32)]), Some(key(30))));
        let metas = window.account_metas();
        assert_eq!(metas.len(), 27);
        assert_eq!(metas.get(22).unwrap().pubkey, key(31));
        assert!(metas.get(22).unwrap().is_writable);
        assert_eq!(metas.get(23).unwrap().pubkey, key(32));
        assert!(metas.get(23).unwrap().is_writable);
        assert_eq!(metas.get(24).unwrap().pubkey, key(30));
        assert!(!metas.get(24).unwrap().is_writable);
        assert_eq!(metas.get(25).unwrap().pubkey, key(20));
        assert!(!metas.get(25).unwrap().is_writable);
        assert_eq!(metas.get(26).unwrap().pubkey, key(21));
        assert!(metas.get(26).unwrap().is_writable);
    }
}
