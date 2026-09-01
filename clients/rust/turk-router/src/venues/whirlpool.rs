//! Orca Whirlpool `swap_v2` windows.
//!
//! The program derives the tick-array tail from the declared account count; this module computes
//! the count from the arrays passed, so the two cannot disagree.
//!
//! # Window
//!
//! `account_count` is `16` with no supplemental tick arrays, and grows by one per array up to
//! `19` with all three.
//!
//! # Slots
//!
//! 1. `[0]` program (readonly)
//! 2. `[1]` token program A (readonly)
//! 3. `[2]` token program B (readonly)
//! 4. `[3]` Memo program (readonly)
//! 5. `[4]` token authority (signer, readonly)
//! 6. `[5]` whirlpool (writable)
//! 7. `[6]` mint A (readonly)
//! 8. `[7]` mint B (readonly)
//! 9. `[8]` user token account A (writable)
//! 10. `[9]` vault A (writable)
//! 11. `[10]` user token account B (writable)
//! 12. `[11]` vault B (writable)
//! 13. `[12]` tick array 0 (writable)
//! 14. `[13]` tick array 1 (writable)
//! 15. `[14]` tick array 2 (writable)
//! 16. `[15]` oracle (writable)
//!
//! # Variable tail
//!
//! `[16..=18]` zero to three supplemental tick arrays (writable), via [`SupplementalTickArrays`].
//!
//! # Token programs
//!
//! Token program A and token program B are read from the caller, independently: each is either
//! the Token program or the Token Extensions program.

use solana_pubkey::Pubkey;

use crate::programs::MEMO_PROGRAM_ID;
use crate::venues::{readonly, signer, writable, VenueWindow};
use crate::HopKind;

/// `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc` — the Orca Whirlpool program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    14, 3, 104, 95, 142, 144, 144, 83, 228, 88, 18, 28, 102, 245, 167, 106, 237, 199, 112, 106,
    161, 28, 130, 248, 170, 149, 42, 143, 43, 120, 121, 169,
]);

const BASE_LEN: u8 = 16;

/// The caller-supplied accounts for one Whirlpool `swap_v2` window. The Memo program and the
/// venue's own program id are fixed by this module, not named here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WhirlpoolAccounts {
    /// `[1]` — readonly. The Token program (or Token Extensions program) that owns mint A.
    pub token_program_a: Pubkey,
    /// `[2]` — readonly. The Token program (or Token Extensions program) that owns mint B.
    pub token_program_b: Pubkey,
    /// `[4]` — signer, readonly. The wallet whose tokens the swap moves.
    pub token_authority: Pubkey,
    /// `[5]` — writable. The pool.
    pub whirlpool: Pubkey,
    /// `[6]` — readonly. The pool's mint A.
    pub mint_a: Pubkey,
    /// `[7]` — readonly. The pool's mint B.
    pub mint_b: Pubkey,
    /// `[8]` — writable. The token authority's account for mint A.
    pub token_owner_account_a: Pubkey,
    /// `[9]` — writable. The pool's vault for mint A.
    pub token_vault_a: Pubkey,
    /// `[10]` — writable. The token authority's account for mint B.
    pub token_owner_account_b: Pubkey,
    /// `[11]` — writable. The pool's vault for mint B.
    pub token_vault_b: Pubkey,
    /// `[12]` — writable. The pool's first tick array for this swap.
    pub tick_array_0: Pubkey,
    /// `[13]` — writable. The pool's second tick array for this swap.
    pub tick_array_1: Pubkey,
    /// `[14]` — writable. The pool's third tick array for this swap.
    pub tick_array_2: Pubkey,
    /// `[15]` — writable. The pool's oracle account.
    pub oracle: Pubkey,
}

/// Zero to three supplemental tick arrays, appended after the base window's fixed 16 accounts.
/// Built only from a fixed-size array: `[Pubkey; 4]` has no [`From`] impl, so a caller cannot
/// hand this type more than the venue accepts.
///
/// ```compile_fail
/// use turk_router::solana_pubkey::Pubkey;
/// use turk_router::venues::whirlpool::SupplementalTickArrays;
///
/// let key = |byte: u8| Pubkey::new_from_array([byte; 32]);
/// let extra: SupplementalTickArrays = SupplementalTickArrays::from([key(1), key(2), key(3), key(4)]);
/// ```
///
/// ```
/// use turk_router::solana_pubkey::Pubkey;
/// use turk_router::venues::whirlpool::{resolve, SupplementalTickArrays, WhirlpoolAccounts};
///
/// let key = |byte: u8| Pubkey::new_from_array([byte; 32]);
/// let accounts = WhirlpoolAccounts {
///     token_program_a: key(1),
///     token_program_b: key(2),
///     token_authority: key(3),
///     whirlpool: key(4),
///     mint_a: key(5),
///     mint_b: key(6),
///     token_owner_account_a: key(7),
///     token_vault_a: key(8),
///     token_owner_account_b: key(9),
///     token_vault_b: key(10),
///     tick_array_0: key(11),
///     tick_array_1: key(12),
///     tick_array_2: key(13),
///     oracle: key(14),
/// };
/// let extra: SupplementalTickArrays = SupplementalTickArrays::from([key(15), key(16), key(17)]);
/// let window = resolve(&accounts, extra);
/// assert_eq!(window.account_count(), 19);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SupplementalTickArrays {
    len: u8,
    keys: [Pubkey; 3],
}

impl SupplementalTickArrays {
    /// How many tick arrays the tail holds, `0..=3`.
    #[must_use]
    pub const fn len(&self) -> u8 {
        self.len
    }

    /// Whether the tail is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The tick arrays, in order.
    #[must_use]
    pub fn keys(&self) -> &[Pubkey] {
        self.keys.get(..usize::from(self.len)).unwrap_or(&[])
    }
}

impl From<[Pubkey; 0]> for SupplementalTickArrays {
    fn from(_: [Pubkey; 0]) -> Self {
        Self {
            len: 0,
            keys: [Pubkey::default(); 3],
        }
    }
}

impl From<[Pubkey; 1]> for SupplementalTickArrays {
    fn from(keys: [Pubkey; 1]) -> Self {
        let [a] = keys;
        Self {
            len: 1,
            keys: [a, Pubkey::default(), Pubkey::default()],
        }
    }
}

impl From<[Pubkey; 2]> for SupplementalTickArrays {
    fn from(keys: [Pubkey; 2]) -> Self {
        let [a, b] = keys;
        Self {
            len: 2,
            keys: [a, b, Pubkey::default()],
        }
    }
}

impl From<[Pubkey; 3]> for SupplementalTickArrays {
    fn from(keys: [Pubkey; 3]) -> Self {
        Self { len: 3, keys }
    }
}

/// Builds the window: the fixed 16 accounts, then `supplemental`'s tick arrays.
#[must_use]
pub fn resolve(accounts: &WhirlpoolAccounts, supplemental: SupplementalTickArrays) -> VenueWindow {
    let account_count = BASE_LEN.saturating_add(supplemental.len());
    let mut metas = Vec::with_capacity(usize::from(account_count));
    metas.push(readonly(PROGRAM_ID));
    metas.push(readonly(accounts.token_program_a));
    metas.push(readonly(accounts.token_program_b));
    metas.push(readonly(MEMO_PROGRAM_ID));
    metas.push(signer(accounts.token_authority));
    metas.push(writable(accounts.whirlpool));
    metas.push(readonly(accounts.mint_a));
    metas.push(readonly(accounts.mint_b));
    metas.push(writable(accounts.token_owner_account_a));
    metas.push(writable(accounts.token_vault_a));
    metas.push(writable(accounts.token_owner_account_b));
    metas.push(writable(accounts.token_vault_b));
    metas.push(writable(accounts.tick_array_0));
    metas.push(writable(accounts.tick_array_1));
    metas.push(writable(accounts.tick_array_2));
    metas.push(writable(accounts.oracle));
    for key in supplemental.keys() {
        metas.push(writable(*key));
    }

    VenueWindow::new(HopKind::Whirlpool, account_count, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts() -> WhirlpoolAccounts {
        WhirlpoolAccounts {
            token_program_a: key(1),
            token_program_b: key(2),
            token_authority: key(3),
            whirlpool: key(4),
            mint_a: key(5),
            mint_b: key(6),
            token_owner_account_a: key(7),
            token_vault_a: key(8),
            token_owner_account_b: key(9),
            token_vault_b: key(10),
            tick_array_0: key(11),
            tick_array_1: key(12),
            tick_array_2: key(13),
            oracle: key(14),
        }
    }

    #[test]
    fn program_id_is_the_documented_address() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
        );
    }

    #[test]
    fn account_count_grows_with_the_supplemental_tail() {
        let accounts = accounts();
        assert_eq!(
            resolve(&accounts, SupplementalTickArrays::from([])).account_count(),
            16
        );
        assert_eq!(
            resolve(&accounts, SupplementalTickArrays::from([key(20)])).account_count(),
            17
        );
        assert_eq!(
            resolve(&accounts, SupplementalTickArrays::from([key(20), key(21)])).account_count(),
            18
        );
        assert_eq!(
            resolve(
                &accounts,
                SupplementalTickArrays::from([key(20), key(21), key(22)])
            )
            .account_count(),
            19
        );
    }

    #[test]
    fn slot_zero_is_the_program_readonly() {
        let window = resolve(&accounts(), SupplementalTickArrays::from([]));
        let program = &window.account_metas()[0];
        assert_eq!(program.pubkey, PROGRAM_ID);
        assert!(!program.is_writable && !program.is_signer);
    }

    #[test]
    fn the_memo_program_is_fixed_not_caller_supplied() {
        let window = resolve(&accounts(), SupplementalTickArrays::from([]));
        assert_eq!(window.account_metas()[3].pubkey, MEMO_PROGRAM_ID);
    }

    #[test]
    fn the_token_authority_slot_is_signer_and_readonly() {
        let window = resolve(&accounts(), SupplementalTickArrays::from([]));
        let authority = &window.account_metas()[4];
        assert!(authority.is_signer);
        assert!(!authority.is_writable);
    }

    #[test]
    fn supplemental_tick_arrays_hold_their_given_length() {
        assert_eq!(SupplementalTickArrays::from([]).len(), 0);
        assert!(SupplementalTickArrays::from([]).is_empty());
        assert_eq!(SupplementalTickArrays::from([key(1)]).len(), 1);
        assert_eq!(SupplementalTickArrays::from([key(1), key(2)]).len(), 2);
        assert_eq!(
            SupplementalTickArrays::from([key(1), key(2), key(3)]).len(),
            3
        );
        assert_eq!(
            SupplementalTickArrays::from([key(1), key(2), key(3)]).keys(),
            &[key(1), key(2), key(3)]
        );
    }
}
