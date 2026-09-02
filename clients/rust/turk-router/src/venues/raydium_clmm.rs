//! Raydium CLMM (`swap_v2`).
//!
//! # Window
//!
//! `account_count` is `15..=21`: 14 fixed slots plus a variable tail of `1..=7` accounts.
//!
//! # Slots
//!
//! 0. Program (readonly)
//! 1. `payer` (signer)
//! 2. `amm_config` (readonly)
//! 3. `pool` (writable)
//! 4. `input_token_account` (writable)
//! 5. `output_token_account` (writable)
//! 6. `input_vault` (writable)
//! 7. `output_vault` (writable)
//! 8. `observation_state` (writable)
//! 9. Token program (readonly)
//! 10. Token Extensions program (readonly)
//! 11. Memo program (readonly)
//! 12. `input_mint` (readonly)
//! 13. `output_mint` (readonly)
//! 14. Tail (writable), `1..=7` accounts — see "Variable tail" below.
//!
//! # Variable tail
//!
//! The optional tick-array bitmap extension, if the pool has one, followed by the tick arrays the
//! swap will cross, one to six.
//!
//! # Token programs
//!
//! Slots 9 and 10 are fixed: the Token program and the Token Extensions program are both present
//! in every `swap_v2` instruction, regardless of which standard `input_mint` and `output_mint`
//! use.

use solana_pubkey::Pubkey;

use crate::programs::{MEMO_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID};
use crate::venues::{readonly, signer, writable, PubkeyTail, VenueWindow};
use crate::HopKind;

/// `CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK` — the Raydium CLMM program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    165, 213, 202, 158, 4, 207, 93, 181, 144, 183, 20, 186, 47, 227, 44, 177, 89, 19, 63, 193, 193,
    146, 183, 34, 87, 253, 7, 211, 156, 176, 64, 30,
]);

const FIXED_SLOTS: u8 = 14;
const TAIL_MAX: usize = 7;

/// The pool's variable tail: the optional tick-array bitmap extension, if the pool has one,
/// followed by the tick arrays the swap will cross, one to six.
pub type ClmmTail = PubkeyTail<TAIL_MAX>;

/// The accounts a Raydium CLMM `swap_v2` takes, named in window order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaydiumClmmAccounts {
    /// The signer whose token accounts the swap moves through.
    pub payer: Pubkey,
    /// The pool's fee and tick-spacing configuration account.
    pub amm_config: Pubkey,
    /// The pool state account.
    pub pool: Pubkey,
    /// `payer`'s token account for `input_mint`.
    pub input_token_account: Pubkey,
    /// `payer`'s token account for `output_mint`.
    pub output_token_account: Pubkey,
    /// The pool's vault for `input_mint`.
    pub input_vault: Pubkey,
    /// The pool's vault for `output_mint`.
    pub output_vault: Pubkey,
    /// The pool's price-observation account.
    pub observation_state: Pubkey,
    /// The mint the swap spends.
    pub input_mint: Pubkey,
    /// The mint the swap receives.
    pub output_mint: Pubkey,
}

/// Builds the window for a Raydium CLMM `swap_v2`.
#[must_use]
pub fn resolve(accounts: RaydiumClmmAccounts, tail: ClmmTail) -> VenueWindow {
    let mut metas = Vec::with_capacity(usize::from(FIXED_SLOTS).saturating_add(tail.len()));
    metas.push(readonly(PROGRAM_ID));
    metas.push(signer(accounts.payer));
    metas.push(readonly(accounts.amm_config));
    metas.push(writable(accounts.pool));
    metas.push(writable(accounts.input_token_account));
    metas.push(writable(accounts.output_token_account));
    metas.push(writable(accounts.input_vault));
    metas.push(writable(accounts.output_vault));
    metas.push(writable(accounts.observation_state));
    metas.push(readonly(TOKEN_PROGRAM_ID));
    metas.push(readonly(TOKEN_2022_PROGRAM_ID));
    metas.push(readonly(MEMO_PROGRAM_ID));
    metas.push(readonly(accounts.input_mint));
    metas.push(readonly(accounts.output_mint));

    let mut tail_len: u8 = 0;
    for key in tail.keys() {
        metas.push(writable(*key));
        tail_len = tail_len.saturating_add(1);
    }

    let account_count = FIXED_SLOTS.saturating_add(tail_len);
    VenueWindow::new(HopKind::RaydiumClmm, account_count, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts() -> RaydiumClmmAccounts {
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

    #[test]
    fn the_program_id_is_the_documented_one() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK"
        );
    }

    #[test]
    fn account_count_is_fourteen_plus_the_tail_length() {
        for len in 1..=TAIL_MAX {
            let mut tail = ClmmTail::new(key(100));
            for extra in 1..len {
                let byte = u8::try_from(extra).unwrap();
                tail.push(key(100 + byte)).unwrap();
            }
            let window = resolve(accounts(), tail);
            let expected = u8::try_from(14usize + len).unwrap();
            assert_eq!(window.account_count(), expected, "tail len {len}");
            assert_eq!(window.account_metas().len(), usize::from(expected));
            assert_eq!(window.hop_kind(), HopKind::RaydiumClmm);
        }
    }

    #[test]
    fn slot_zero_is_the_program_readonly() {
        let window = resolve(accounts(), ClmmTail::new(key(50)));
        let program = &window.account_metas()[0];
        assert_eq!(program.pubkey, PROGRAM_ID);
        assert!(!program.is_writable && !program.is_signer);
    }

    #[test]
    fn the_payer_is_the_only_signer() {
        let window = resolve(accounts(), ClmmTail::new(key(50)));
        let payer = &window.account_metas()[1];
        assert_eq!(payer.pubkey, key(1));
        assert!(payer.is_signer);
        assert!(!payer.is_writable);
    }
}
