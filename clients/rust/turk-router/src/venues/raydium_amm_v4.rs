//! Raydium AMM v4's `swap_base_in` window.
//!
//! # Window
//!
//! `account_count` is always 9.
//!
//! # Slots
//!
//! 0. Program (readonly): [`PROGRAM_ID`].
//! 1. Token program (readonly): the classic Token program.
//! 2. Pool (writable): [`RaydiumAmmV4Accounts::pool`].
//! 3. AMM authority (readonly): [`AMM_AUTHORITY`].
//! 4. Base vault (writable): [`RaydiumAmmV4Accounts::base_vault`].
//! 5. Quote vault (writable): [`RaydiumAmmV4Accounts::quote_vault`].
//! 6. User source (writable): [`RaydiumAmmV4Accounts::user_source`].
//! 7. User destination (writable): [`RaydiumAmmV4Accounts::user_destination`].
//! 8. Payer (signer): [`RaydiumAmmV4Accounts::payer`].
//!
//! # Variable tail
//!
//! None: every window carries exactly 9 accounts.
//!
//! # Token programs
//!
//! The classic Token program only.

use solana_pubkey::Pubkey;

use super::{readonly, signer, writable, VenueWindow};
use crate::programs::TOKEN_PROGRAM_ID;
use crate::HopKind;

/// `675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8` — the Raydium AMM v4 program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    75, 217, 73, 196, 54, 2, 195, 63, 32, 119, 144, 237, 22, 163, 82, 76, 161, 185, 151, 92, 241,
    33, 162, 169, 12, 255, 236, 125, 248, 182, 138, 205,
]);

/// `5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1` — the pool authority PDA every Raydium AMM v4
/// pool shares.
pub const AMM_AUTHORITY: Pubkey = Pubkey::new_from_array([
    65, 87, 176, 88, 15, 49, 197, 252, 228, 74, 98, 88, 45, 188, 249, 215, 142, 231, 89, 67, 160,
    132, 163, 147, 179, 80, 54, 141, 34, 137, 147, 8,
]);

const ACCOUNT_COUNT: u8 = 9;

/// The accounts a caller supplies for one Raydium AMM v4 hop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaydiumAmmV4Accounts {
    /// The pool account, owned by [`PROGRAM_ID`].
    pub pool: Pubkey,
    /// The pool's base token vault.
    pub base_vault: Pubkey,
    /// The pool's quote token vault.
    pub quote_vault: Pubkey,
    /// The caller's token account this hop debits.
    pub user_source: Pubkey,
    /// The caller's token account this hop credits.
    pub user_destination: Pubkey,
    /// The wallet authorizing the debit from [`Self::user_source`].
    pub payer: Pubkey,
}

/// Builds the window for one Raydium AMM v4 hop.
#[must_use]
pub fn resolve(accounts: RaydiumAmmV4Accounts) -> VenueWindow {
    let metas = vec![
        readonly(PROGRAM_ID),
        readonly(TOKEN_PROGRAM_ID),
        writable(accounts.pool),
        readonly(AMM_AUTHORITY),
        writable(accounts.base_vault),
        writable(accounts.quote_vault),
        writable(accounts.user_source),
        writable(accounts.user_destination),
        signer(accounts.payer),
    ];
    VenueWindow::new(HopKind::RaydiumAmmV4, ACCOUNT_COUNT, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    #[test]
    fn program_id_is_the_documented_address() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"
        );
    }

    #[test]
    fn amm_authority_is_the_documented_address() {
        assert_eq!(
            AMM_AUTHORITY.to_string(),
            "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"
        );
    }

    #[test]
    fn the_window_always_declares_nine_accounts() {
        let window = resolve(RaydiumAmmV4Accounts {
            pool: key(1),
            base_vault: key(2),
            quote_vault: key(3),
            user_source: key(4),
            user_destination: key(5),
            payer: key(6),
        });
        assert_eq!(window.account_count(), 9);
        assert_eq!(window.account_metas().len(), 9);
        assert_eq!(window.hop_kind(), HopKind::RaydiumAmmV4);
    }

    #[test]
    fn the_slots_carry_the_documented_flags_in_order() {
        let window = resolve(RaydiumAmmV4Accounts {
            pool: key(1),
            base_vault: key(2),
            quote_vault: key(3),
            user_source: key(4),
            user_destination: key(5),
            payer: key(6),
        });
        let metas = window.account_metas();

        assert_eq!(metas[0].pubkey, PROGRAM_ID);
        assert!(!metas[0].is_writable && !metas[0].is_signer);

        assert_eq!(metas[1].pubkey, TOKEN_PROGRAM_ID);
        assert!(!metas[1].is_writable && !metas[1].is_signer);

        assert_eq!(metas[2].pubkey, key(1));
        assert!(metas[2].is_writable && !metas[2].is_signer);

        assert_eq!(metas[3].pubkey, AMM_AUTHORITY);
        assert!(!metas[3].is_writable && !metas[3].is_signer);

        assert_eq!(metas[4].pubkey, key(2));
        assert!(metas[4].is_writable && !metas[4].is_signer);

        assert_eq!(metas[5].pubkey, key(3));
        assert!(metas[5].is_writable && !metas[5].is_signer);

        assert_eq!(metas[6].pubkey, key(4));
        assert!(metas[6].is_writable && !metas[6].is_signer);

        assert_eq!(metas[7].pubkey, key(5));
        assert!(metas[7].is_writable && !metas[7].is_signer);

        assert_eq!(metas[8].pubkey, key(6));
        assert!(!metas[8].is_writable && metas[8].is_signer);
    }
}
