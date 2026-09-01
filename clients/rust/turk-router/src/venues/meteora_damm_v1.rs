//! Meteora DAMM v1 (Dynamic AMM), the `swap` instruction.
//!
//! # Window
//!
//! `account_count` is always 16.
//!
//! # Slots
//!
//! 0. venue program (readonly) — [`PROGRAM_ID`]
//! 1. `pool` (writable)
//! 2. `user_source` (writable)
//! 3. `user_dest` (writable)
//! 4. `a_vault` (writable)
//! 5. `b_vault` (writable)
//! 6. `a_token_vault` (writable)
//! 7. `b_token_vault` (writable)
//! 8. `a_vault_lp_mint` (writable)
//! 9. `b_vault_lp_mint` (writable)
//! 10. `a_vault_lp` (writable)
//! 11. `b_vault_lp` (writable)
//! 12. `protocol_token_fee` (writable)
//! 13. `payer` (signer)
//! 14. Dynamic Vault program (readonly) — [`DYNAMIC_VAULT_PROGRAM_ID`]
//! 15. token program (readonly) — [`crate::programs::TOKEN_PROGRAM_ID`]
//!
//! # Variable tail
//!
//! None: every window carries exactly 16 accounts.
//!
//! # Token programs
//!
//! The classic Token program only.

use solana_pubkey::Pubkey;

use crate::programs::TOKEN_PROGRAM_ID;
use crate::venues::{readonly, signer, writable, VenueWindow};
use crate::HopKind;

/// `Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB` — the Meteora DAMM v1 program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    204, 248, 2, 212, 204, 204, 132, 215, 251, 33, 181, 247, 59, 73, 216, 26, 22, 197, 180, 200,
    142, 227, 35, 148, 225, 201, 29, 53, 136, 204, 64, 128,
]);

/// `24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi` — the Meteora Dynamic Vault program every DAMM
/// v1 swap names, one slot after the payer.
pub const DYNAMIC_VAULT_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    15, 191, 232, 132, 109, 104, 92, 189, 198, 44, 202, 126, 4, 199, 232, 246, 141, 204, 49, 58,
    179, 18, 119, 226, 224, 17, 42, 46, 192, 224, 82, 229,
]);

const ACCOUNT_COUNT: u8 = 16;

/// The caller-supplied accounts for one Meteora DAMM v1 `swap` hop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeteoraDammV1Accounts {
    /// The pool.
    pub pool: Pubkey,
    /// The caller's token account this hop debits.
    pub user_source: Pubkey,
    /// The caller's token account this hop credits.
    pub user_dest: Pubkey,
    /// The pool's vault for its token-A side.
    pub a_vault: Pubkey,
    /// The pool's vault for its token-B side.
    pub b_vault: Pubkey,
    /// Token-A vault's own token account holding the reserve.
    pub a_token_vault: Pubkey,
    /// Token-B vault's own token account holding the reserve.
    pub b_token_vault: Pubkey,
    /// Token-A vault's LP mint.
    pub a_vault_lp_mint: Pubkey,
    /// Token-B vault's LP mint.
    pub b_vault_lp_mint: Pubkey,
    /// The pool's LP token account in token-A vault's LP mint.
    pub a_vault_lp: Pubkey,
    /// The pool's LP token account in token-B vault's LP mint.
    pub b_vault_lp: Pubkey,
    /// The pool's protocol fee token account for the side being sold.
    pub protocol_token_fee: Pubkey,
    /// The wallet authorizing the debit from [`Self::user_source`].
    pub payer: Pubkey,
}

/// Builds the window for one Meteora DAMM v1 hop.
#[must_use]
pub fn resolve(accounts: MeteoraDammV1Accounts) -> VenueWindow {
    let metas = vec![
        readonly(PROGRAM_ID),
        writable(accounts.pool),
        writable(accounts.user_source),
        writable(accounts.user_dest),
        writable(accounts.a_vault),
        writable(accounts.b_vault),
        writable(accounts.a_token_vault),
        writable(accounts.b_token_vault),
        writable(accounts.a_vault_lp_mint),
        writable(accounts.b_vault_lp_mint),
        writable(accounts.a_vault_lp),
        writable(accounts.b_vault_lp),
        writable(accounts.protocol_token_fee),
        signer(accounts.payer),
        readonly(DYNAMIC_VAULT_PROGRAM_ID),
        readonly(TOKEN_PROGRAM_ID),
    ];
    VenueWindow::new(HopKind::MeteoraDammV1, ACCOUNT_COUNT, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts() -> MeteoraDammV1Accounts {
        MeteoraDammV1Accounts {
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
        }
    }

    #[test]
    fn the_addresses_are_the_documented_ones() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB"
        );
        assert_eq!(
            DYNAMIC_VAULT_PROGRAM_ID.to_string(),
            "24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi"
        );
    }

    #[test]
    fn the_window_always_declares_sixteen_accounts() {
        let window = resolve(accounts());
        assert_eq!(window.account_count(), 16);
        assert_eq!(window.account_metas().len(), 16);
        assert_eq!(window.hop_kind(), HopKind::MeteoraDammV1);
    }

    #[test]
    fn the_slots_carry_the_documented_flags_in_order() {
        let window = resolve(accounts());
        let metas = window.account_metas();

        let expected_pubkeys = [
            PROGRAM_ID,
            key(1),
            key(2),
            key(3),
            key(4),
            key(5),
            key(6),
            key(7),
            key(8),
            key(9),
            key(10),
            key(11),
            key(12),
            key(13),
            DYNAMIC_VAULT_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
        ];
        let writable_slots = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        for (index, meta) in metas.iter().enumerate() {
            assert_eq!(meta.pubkey, expected_pubkeys[index], "slot {index}");
            assert_eq!(
                meta.is_writable,
                writable_slots.contains(&index),
                "slot {index}"
            );
            assert_eq!(meta.is_signer, index == 13, "slot {index}");
        }
    }
}
