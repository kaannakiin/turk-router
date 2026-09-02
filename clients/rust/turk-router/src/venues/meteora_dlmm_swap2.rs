//! Meteora DLMM, the `swap2` instruction — the DLMM swap form that accepts a Token-2022 mint on
//! either side of the pair.
//!
//! # Window
//!
//! `account_count` is `18..=25`: 17 fixed slots (`[0]` through `[16]`, the venue program included
//! twice) plus the bin array tail's own length, `1..=8`.
//!
//! # Slots
//!
//! - `[0]` [`PROGRAM_ID`] (readonly) — the DLMM program.
//! - `[1]` `pool` (writable) — the `LbPair` account this hop trades against.
//! - `[2]` `bin_array_bitmap_extension` (writable when given; readonly [`PROGRAM_ID`] sentinel
//!   when the pair's bin range does not need one).
//! - `[3]` `reserve_x` (writable) — the pair's reserve for `token_x`.
//! - `[4]` `reserve_y` (writable) — the pair's reserve for `token_y`.
//! - `[5]` `user_token_in` (writable) — the caller's token account the hop debits.
//! - `[6]` `user_token_out` (writable) — the caller's token account the hop credits.
//! - `[7]` `token_x_mint` (readonly).
//! - `[8]` `token_y_mint` (readonly).
//! - `[9]` `oracle` (writable) — the pair's oracle account.
//! - `[10]` `host_fee_in` (writable), or the readonly [`PROGRAM_ID`] sentinel when the caller names
//!   none.
//! - `[11]` `user` (signer) — the wallet authorizing the swap.
//! - `[12]` `token_x_program` (readonly).
//! - `[13]` `token_y_program` (readonly).
//! - `[14]` the Memo program (readonly, [`crate::programs::MEMO_PROGRAM_ID`]).
//! - `[15]` the event-authority PDA (readonly, fixed).
//! - `[16]` [`PROGRAM_ID`] again (readonly).
//! - `[17..=24]` the bin array tail (writable), `1..=8` accounts.
//!
//! # Variable tail
//!
//! `bin_arrays` carries `1..=8` accounts (a [`PubkeyTail`]), each writable, appended in order
//! after slot `[16]`.
//!
//! # Token programs
//!
//! `token_x_program` and `token_y_program` are caller-supplied and independent of each other:
//! either may be the Token program or the Token Extensions (Token-2022) program.
//!
//! # Transfer hooks
//!
//! The program accepts transfer-hook account groups on this kind. This module builds none — the
//! window it returns never carries one.

use solana_pubkey::Pubkey;

use super::{readonly, signer, writable, PubkeyTail, VenueWindow};
use crate::programs::MEMO_PROGRAM_ID;
use crate::HopKind;

/// `LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo` — the Meteora DLMM program.
pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    4, 233, 225, 47, 188, 132, 232, 38, 201, 50, 204, 233, 226, 100, 12, 206, 21, 89, 12, 28, 98,
    115, 176, 146, 87, 8, 186, 59, 133, 32, 176, 188,
]);

/// `D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6` — the program's event-authority PDA.
const EVENT_AUTHORITY: Pubkey = Pubkey::new_from_array([
    178, 112, 214, 127, 169, 140, 81, 207, 2, 19, 5, 19, 88, 150, 43, 175, 53, 116, 43, 237, 89,
    201, 217, 68, 94, 156, 13, 12, 133, 199, 205, 145,
]);

/// Most bin arrays a `swap2` window's variable tail carries.
pub const MAX_BIN_ARRAYS: usize = 8;

/// Window accounts before the bin array tail, slot `[0]` included.
const FIXED_ACCOUNTS: u8 = 17;

/// The caller-supplied accounts for a Meteora DLMM `swap2` window, in window order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeteoraDlmmSwap2Accounts {
    /// The `LbPair` account this hop trades against.
    pub pool: Pubkey,
    /// The pair's bin array bitmap extension, when its active bin range needs one.
    pub bin_array_bitmap_extension: Option<Pubkey>,
    /// The pair's reserve for `token_x`.
    pub reserve_x: Pubkey,
    /// The pair's reserve for `token_y`.
    pub reserve_y: Pubkey,
    /// The caller's token account the hop debits.
    pub user_token_in: Pubkey,
    /// The caller's token account the hop credits.
    pub user_token_out: Pubkey,
    /// The pair's `token_x` mint.
    pub token_x_mint: Pubkey,
    /// The pair's `token_y` mint.
    pub token_y_mint: Pubkey,
    /// The pair's oracle account.
    pub oracle: Pubkey,
    /// The host fee token account, when the caller names one.
    pub host_fee_in: Option<Pubkey>,
    /// The wallet authorizing the swap.
    pub user: Pubkey,
    /// The token program owning `token_x_mint` — Token or Token-2022.
    pub token_x_program: Pubkey,
    /// The token program owning `token_y_mint` — Token or Token-2022.
    pub token_y_program: Pubkey,
}

#[expect(
    clippy::expect_used,
    reason = "bin_arrays.len() is bounded to 1..=MAX_BIN_ARRAYS by PubkeyTail's own constructors, \
              and FIXED_ACCOUNTS (17) plus at most 8 always fits u8"
)]
fn account_count(bin_arrays_len: usize) -> u8 {
    let bins = u8::try_from(bin_arrays_len).expect("bin tail length fits u8");
    FIXED_ACCOUNTS
        .checked_add(bins)
        .expect("17 + bins (<=8) fits u8")
}

/// Builds the window for one Meteora DLMM `swap2` pool.
///
/// Infallible: every fixed account is a plain [`Pubkey`], and `bin_arrays`'s length is already
/// bounded by [`PubkeyTail`]'s own constructors.
#[must_use]
pub fn resolve(
    accounts: MeteoraDlmmSwap2Accounts,
    bin_arrays: PubkeyTail<MAX_BIN_ARRAYS>,
) -> VenueWindow {
    let MeteoraDlmmSwap2Accounts {
        pool,
        bin_array_bitmap_extension,
        reserve_x,
        reserve_y,
        user_token_in,
        user_token_out,
        token_x_mint,
        token_y_mint,
        oracle,
        host_fee_in,
        user,
        token_x_program,
        token_y_program,
    } = accounts;

    let count = account_count(bin_arrays.len());
    let mut metas = Vec::with_capacity(usize::from(count));
    metas.push(readonly(PROGRAM_ID));
    metas.push(writable(pool));
    metas.push(match bin_array_bitmap_extension {
        Some(key) => writable(key),
        None => readonly(PROGRAM_ID),
    });
    metas.push(writable(reserve_x));
    metas.push(writable(reserve_y));
    metas.push(writable(user_token_in));
    metas.push(writable(user_token_out));
    metas.push(readonly(token_x_mint));
    metas.push(readonly(token_y_mint));
    metas.push(writable(oracle));
    metas.push(host_fee_in.map_or_else(|| readonly(PROGRAM_ID), writable));
    metas.push(signer(user));
    metas.push(readonly(token_x_program));
    metas.push(readonly(token_y_program));
    metas.push(readonly(MEMO_PROGRAM_ID));
    metas.push(readonly(EVENT_AUTHORITY));
    metas.push(readonly(PROGRAM_ID));
    for key in bin_arrays.keys() {
        metas.push(writable(*key));
    }

    VenueWindow::new(HopKind::MeteoraDlmmSwap2, count, metas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    fn bin_pool() -> [Pubkey; MAX_BIN_ARRAYS] {
        [
            key(20),
            key(21),
            key(22),
            key(23),
            key(24),
            key(25),
            key(26),
            key(27),
        ]
    }

    fn sample_accounts() -> MeteoraDlmmSwap2Accounts {
        MeteoraDlmmSwap2Accounts {
            pool: key(1),
            bin_array_bitmap_extension: None,
            reserve_x: key(2),
            reserve_y: key(3),
            user_token_in: key(4),
            user_token_out: key(5),
            token_x_mint: key(6),
            token_y_mint: key(7),
            oracle: key(8),
            host_fee_in: None,
            user: key(9),
            token_x_program: key(10),
            token_y_program: key(11),
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
    fn account_count_spans_the_bin_tail_range() {
        let pool = bin_pool();
        for (len, expected) in (1..=MAX_BIN_ARRAYS).zip(18u8..=25u8) {
            let bin_arrays = PubkeyTail::<MAX_BIN_ARRAYS>::try_from_slice(&pool[..len]).unwrap();
            let window = resolve(sample_accounts(), bin_arrays);
            assert_eq!(window.account_count(), expected);
            assert_eq!(window.account_metas().len(), usize::from(expected));
        }
    }

    #[test]
    fn the_optional_slots_do_not_change_account_count() {
        let bin_arrays = PubkeyTail::<MAX_BIN_ARRAYS>::new(key(30));
        let bare = resolve(sample_accounts(), bin_arrays).account_count();

        let mut accounts = sample_accounts();
        accounts.bin_array_bitmap_extension = Some(key(31));
        accounts.host_fee_in = Some(key(32));
        assert_eq!(resolve(accounts, bin_arrays).account_count(), bare);
    }

    #[test]
    fn slot_0_and_slot_16_are_the_venue_program_readonly() {
        let bin_arrays = PubkeyTail::<MAX_BIN_ARRAYS>::new(key(40));
        let window = resolve(sample_accounts(), bin_arrays);
        let metas = window.account_metas();

        let first = metas.first().unwrap();
        assert_eq!(first.pubkey, PROGRAM_ID);
        assert!(!first.is_writable && !first.is_signer);

        let program_again = metas.get(16).unwrap();
        assert_eq!(program_again.pubkey, PROGRAM_ID);
        assert!(!program_again.is_writable && !program_again.is_signer);
    }

    #[test]
    fn absent_optionals_fall_back_to_the_program_id_sentinel() {
        let bin_arrays = PubkeyTail::<MAX_BIN_ARRAYS>::new(key(41));
        let window = resolve(sample_accounts(), bin_arrays);
        let metas = window.account_metas();

        let bitmap_extension = metas.get(2).unwrap();
        assert_eq!(bitmap_extension.pubkey, PROGRAM_ID);
        assert!(!bitmap_extension.is_writable);

        let host_fee_in = metas.get(10).unwrap();
        assert_eq!(host_fee_in.pubkey, PROGRAM_ID);
        assert!(!host_fee_in.is_writable);
    }

    #[test]
    fn a_named_host_fee_account_is_writable() {
        let mut accounts = sample_accounts();
        accounts.host_fee_in = Some(key(32));
        let window = resolve(accounts, crate::venues::PubkeyTail::new(key(50)));
        let slot = window.account_metas().get(10).unwrap();
        assert_eq!(slot.pubkey, key(32));
        assert!(slot.is_writable);
        assert!(!slot.is_signer);
    }
}
