//! Meteora DLMM's `swap` instruction — the classic form, pinned to the Token program on both
//! sides of the pool.
//!
//! # Window
//!
//! `account_count` is `17..=24`: a 16-slot fixed prefix (slot 0, the venue program, included)
//! plus a `1..=8`-account bin array tail.
//!
//! # Slots
//!
//! 0. venue program (readonly) — [`PROGRAM_ID`]
//! 1. `lb_pair` (writable)
//! 2. `bin_array_bitmap_extension` (writable), or [`PROGRAM_ID`] (readonly) when the pool carries
//!    none
//! 3. `reserve_x` (writable)
//! 4. `reserve_y` (writable)
//! 5. `user_token_in` (writable)
//! 6. `user_token_out` (writable)
//! 7. `mint_x` (readonly)
//! 8. `mint_y` (readonly)
//! 9. `oracle` (writable)
//! 10. `host_fee_in` (writable), or [`PROGRAM_ID`] (readonly) when absent
//! 11. `user` (signer)
//! 12. `token_x_program` (readonly) — [`crate::programs::TOKEN_PROGRAM_ID`]
//! 13. `token_y_program` (readonly) — [`crate::programs::TOKEN_PROGRAM_ID`]
//! 14. `event_authority` (readonly) — [`EVENT_AUTHORITY`]
//! 15. venue program again (readonly) — [`PROGRAM_ID`]
//!
//! Slot 16 onward is the bin array tail; see below.
//!
//! # Variable tail
//!
//! Slots 16 through `15 + `[`MAX_BINS`] are the bin arrays (writable), `1..=`[`MAX_BINS`]
//! accounts, appended in the order given.
//!
//! # Token programs
//!
//! Both token-program slots are fixed to the Token program; this instruction form accepts no
//! other.

use solana_pubkey::Pubkey;

use crate::programs::TOKEN_PROGRAM_ID;
use crate::venues::{readonly, signer, writable, PubkeyTail, VenueWindow};
use crate::HopKind;

/// `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` — the Meteora DLMM program. Also the sentinel
/// this module writes for `bin_array_bitmap_extension` and `host_fee_in` when the caller has
/// none.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    4, 233, 225, 47, 188, 132, 232, 38, 201, 50, 204, 233, 226, 100, 12, 206, 21, 89, 12, 28, 98,
    115, 176, 146, 87, 8, 186, 59, 133, 32, 176, 188,
]);

/// `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6` — the event-authority PDA this instruction
/// reads.
pub const EVENT_AUTHORITY: Pubkey = Pubkey::new_from_array([
    178, 112, 214, 127, 169, 140, 81, 207, 2, 19, 5, 19, 88, 150, 43, 175, 53, 116, 43, 237, 89,
    201, 217, 68, 94, 156, 13, 12, 133, 199, 205, 145,
]);

/// The bin array tail's longest reach.
pub const MAX_BINS: usize = 8;

/// The bin array tail: `1..=`[`MAX_BINS`] accounts, in the order the instruction reads them.
pub type BinArrayTail = PubkeyTail<MAX_BINS>;

const FIXED_LEN: u8 = 16;

/// The accounts the caller supplies for one pool, in window order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeteoraDlmmSwapAccounts {
    /// The pool.
    pub lb_pair: Pubkey,
    /// The pool's bin array bitmap extension, when it has one.
    pub bin_array_bitmap_extension: Option<Pubkey>,
    /// The pool's reserve for its `x` token.
    pub reserve_x: Pubkey,
    /// The pool's reserve for its `y` token.
    pub reserve_y: Pubkey,
    /// The user's token account the instruction debits.
    pub user_token_in: Pubkey,
    /// The user's token account the instruction credits.
    pub user_token_out: Pubkey,
    /// The pool's `x` mint.
    pub mint_x: Pubkey,
    /// The pool's `y` mint.
    pub mint_y: Pubkey,
    /// The pool's price oracle.
    pub oracle: Pubkey,
    /// The host's fee-collection account, when the pool has a host fee configured.
    pub host_fee_in: Option<Pubkey>,
    /// The signer whose tokens move.
    pub user: Pubkey,
}

/// Builds the window for one pool.
#[must_use]
pub fn resolve(accounts: MeteoraDlmmSwapAccounts, bin_arrays: BinArrayTail) -> VenueWindow {
    let account_count = account_count(&bin_arrays);
    let mut metas = Vec::with_capacity(usize::from(account_count));
    metas.push(readonly(PROGRAM_ID));
    metas.push(writable(accounts.lb_pair));
    metas.push(optional_or_sentinel(accounts.bin_array_bitmap_extension));
    metas.push(writable(accounts.reserve_x));
    metas.push(writable(accounts.reserve_y));
    metas.push(writable(accounts.user_token_in));
    metas.push(writable(accounts.user_token_out));
    metas.push(readonly(accounts.mint_x));
    metas.push(readonly(accounts.mint_y));
    metas.push(writable(accounts.oracle));
    metas.push(optional_or_sentinel(accounts.host_fee_in));
    metas.push(signer(accounts.user));
    metas.push(readonly(TOKEN_PROGRAM_ID));
    metas.push(readonly(TOKEN_PROGRAM_ID));
    metas.push(readonly(EVENT_AUTHORITY));
    metas.push(readonly(PROGRAM_ID));
    for bin_array in bin_arrays.keys() {
        metas.push(writable(*bin_array));
    }

    VenueWindow::new(HopKind::MeteoraDlmmSwap, account_count, metas)
}

fn optional_or_sentinel(account: Option<Pubkey>) -> solana_instruction::AccountMeta {
    account.map_or_else(|| readonly(PROGRAM_ID), writable)
}

#[expect(
    clippy::expect_used,
    reason = "BinArrayTail::len() is bounded to 1..=MAX_BINS (8) by construction, so it always \
              fits u8, and FIXED_LEN (16) plus that always fits u8 as well"
)]
fn account_count(bin_arrays: &BinArrayTail) -> u8 {
    let bins_len = u8::try_from(bin_arrays.len()).expect("bounded to 1..=MAX_BINS");
    FIXED_LEN
        .checked_add(bins_len)
        .expect("FIXED_LEN + MAX_BINS fits in u8")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn accounts() -> MeteoraDlmmSwapAccounts {
        MeteoraDlmmSwapAccounts {
            lb_pair: key(1),
            bin_array_bitmap_extension: None,
            reserve_x: key(2),
            reserve_y: key(3),
            user_token_in: key(4),
            user_token_out: key(5),
            mint_x: key(6),
            mint_y: key(7),
            oracle: key(8),
            host_fee_in: None,
            user: key(9),
        }
    }

    #[test]
    fn program_id_is_the_documented_address() {
        assert_eq!(
            PROGRAM_ID.to_string(),
            "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo"
        );
    }

    #[test]
    fn event_authority_is_the_documented_address() {
        assert_eq!(
            EVENT_AUTHORITY.to_string(),
            "D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6"
        );
    }

    #[test]
    fn account_count_tracks_the_bin_tail_length() {
        let one_bin = BinArrayTail::new(key(10));
        let window = resolve(accounts(), one_bin);
        assert_eq!(window.account_count(), 17);
        assert_eq!(window.account_metas().len(), 17);

        let placeholders = [key(20); MAX_BINS];
        let max_bins =
            BinArrayTail::try_from_slice(&placeholders).expect("MAX_BINS accounts fit the tail");
        let window = resolve(accounts(), max_bins);
        assert_eq!(window.account_count(), 24);
        assert_eq!(window.account_metas().len(), 24);
    }

    #[test]
    fn account_count_does_not_depend_on_the_optional_slots() {
        let bins = BinArrayTail::new(key(10));
        let without_optionals = resolve(accounts(), bins);

        let mut with_optionals = accounts();
        with_optionals.bin_array_bitmap_extension = Some(key(11));
        with_optionals.host_fee_in = Some(key(12));
        let with_optionals = resolve(with_optionals, bins);

        assert_eq!(
            without_optionals.account_count(),
            with_optionals.account_count()
        );
    }

    #[test]
    fn slot_zero_and_the_repeated_program_slot_are_the_venue_program() {
        let window = resolve(accounts(), BinArrayTail::new(key(10)));
        let metas = window.account_metas();
        assert_eq!(metas[0].pubkey, PROGRAM_ID);
        assert!(!metas[0].is_writable && !metas[0].is_signer);
        assert_eq!(metas[15].pubkey, PROGRAM_ID);
        assert!(!metas[15].is_writable && !metas[15].is_signer);
    }

    #[test]
    fn absent_optionals_fall_back_to_the_program_id_sentinel() {
        let window = resolve(accounts(), BinArrayTail::new(key(10)));
        let metas = window.account_metas();
        assert_eq!(metas[2].pubkey, PROGRAM_ID);
        assert!(!metas[2].is_writable);
        assert_eq!(metas[10].pubkey, PROGRAM_ID);
        assert!(!metas[10].is_writable);
    }

    #[test]
    fn present_optionals_are_writable() {
        let mut given = accounts();
        given.bin_array_bitmap_extension = Some(key(11));
        given.host_fee_in = Some(key(12));
        let window = resolve(given, BinArrayTail::new(key(10)));
        let metas = window.account_metas();
        assert_eq!(metas[2].pubkey, key(11));
        assert!(metas[2].is_writable);
        assert_eq!(metas[10].pubkey, key(12));
        assert!(metas[10].is_writable);
    }

    #[test]
    fn both_token_program_slots_are_fixed_to_the_token_program() {
        let window = resolve(accounts(), BinArrayTail::new(key(10)));
        let metas = window.account_metas();
        assert_eq!(metas[12].pubkey, TOKEN_PROGRAM_ID);
        assert_eq!(metas[13].pubkey, TOKEN_PROGRAM_ID);
    }

    #[test]
    fn user_is_a_readonly_signer() {
        let window = resolve(accounts(), BinArrayTail::new(key(10)));
        let user_meta = &window.account_metas()[11];
        assert_eq!(user_meta.pubkey, key(9));
        assert!(user_meta.is_signer);
        assert!(!user_meta.is_writable);
    }
}
