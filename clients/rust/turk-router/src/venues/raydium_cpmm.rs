//! Raydium CPMM.
//!
//! # Window
//!
//! `account_count` is always 14.
//!
//! # Slots
//!
//! 0. Program (readonly)
//! 1. `user` (signer)
//! 2. Swap authority (readonly)
//! 3. `amm_config` (readonly)
//! 4. `pool` (writable)
//! 5. `input_token_account` (writable)
//! 6. `output_token_account` (writable)
//! 7. `input_vault` (writable)
//! 8. `output_vault` (writable)
//! 9. `input_token_program` (readonly)
//! 10. `output_token_program` (readonly)
//! 11. `input_mint` (readonly)
//! 12. `output_mint` (readonly)
//! 13. `observation_state` (writable)
//!
//! # Token programs
//!
//! `input_token_program` and `output_token_program` each independently name the Token program or
//! the Token Extensions program.

use solana_pubkey::Pubkey;

use crate::venues::{readonly, signer, writable, VenueWindow};
use crate::HopKind;

/// `CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C` — the Raydium CPMM program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    169, 42, 90, 139, 79, 41, 89, 82, 132, 37, 80, 170, 147, 253, 91, 149, 181, 172, 230, 168, 235,
    146, 12, 147, 148, 46, 67, 105, 12, 32, 236, 115,
]);

/// `GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL` — the program-wide PDA every pool's swap
/// instruction names as its authority.
pub const AUTHORITY: Pubkey = Pubkey::new_from_array([
    235, 0, 217, 245, 178, 146, 180, 33, 74, 199, 208, 55, 180, 214, 240, 100, 80, 185, 100, 96,
    13, 243, 115, 5, 43, 181, 232, 79, 47, 142, 154, 103,
]);

const ACCOUNT_COUNT: u8 = 14;

/// The accounts a Raydium CPMM swap takes, named in window order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaydiumCpmmAccounts {
    /// The signer whose token accounts the swap moves through.
    pub user: Pubkey,
    /// The pool's fee and protocol configuration account.
    pub amm_config: Pubkey,
    /// The pool state account.
    pub pool: Pubkey,
    /// `user`'s token account for `input_mint`.
    pub input_token_account: Pubkey,
    /// `user`'s token account for `output_mint`.
    pub output_token_account: Pubkey,
    /// The pool's vault for `input_mint`.
    pub input_vault: Pubkey,
    /// The pool's vault for `output_mint`.
    pub output_vault: Pubkey,
    /// The program that owns `input_token_account` and `input_vault`.
    pub input_token_program: Pubkey,
    /// The program that owns `output_token_account` and `output_vault`.
    pub output_token_program: Pubkey,
    /// The mint the swap spends.
    pub input_mint: Pubkey,
    /// The mint the swap receives.
    pub output_mint: Pubkey,
    /// The pool's price-observation account.
    pub observation_state: Pubkey,
}

/// Builds the window for a Raydium CPMM swap.
#[must_use]
pub fn resolve(accounts: RaydiumCpmmAccounts) -> VenueWindow {
    let metas = vec![
        readonly(PROGRAM_ID),
        signer(accounts.user),
        readonly(AUTHORITY),
        readonly(accounts.amm_config),
        writable(accounts.pool),
        writable(accounts.input_token_account),
        writable(accounts.output_token_account),
        writable(accounts.input_vault),
        writable(accounts.output_vault),
        readonly(accounts.input_token_program),
        readonly(accounts.output_token_program),
        readonly(accounts.input_mint),
        readonly(accounts.output_mint),
        writable(accounts.observation_state),
    ];
    VenueWindow::new(HopKind::RaydiumCpmm, ACCOUNT_COUNT, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    #[test]
    fn the_program_id_is_the_documented_one() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C"
        );
    }

    #[test]
    fn the_authority_is_the_documented_one() {
        assert_eq!(
            AUTHORITY.to_string(),
            "GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL"
        );
    }

    #[test]
    fn the_window_declares_fourteen_accounts() {
        let window = resolve(RaydiumCpmmAccounts {
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
        });
        assert_eq!(window.account_count(), 14);
        assert_eq!(window.account_metas().len(), 14);
        assert_eq!(window.hop_kind(), HopKind::RaydiumCpmm);
    }
}
