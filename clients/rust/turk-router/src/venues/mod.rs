//! One module per menu kind. Each exposes a `resolve` that takes the venue's accounts as named
//! fields — plus a bounded type for any variable tail — and returns a [`VenueWindow`] whose
//! declared account count is computed from what it holds, so the two cannot disagree.
//!
//! Slot 0 of every window is the venue's program; the accounts the program's swap instruction
//! takes follow it in the venue's own order. Transfer-hook account groups are not built here:
//! every window is declared with no hook accounts.

use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

use crate::wire::MENU_ENTRY_LEN;
use crate::{Error, HopKind};

pub mod meteora_damm_v1;
pub mod meteora_damm_v2;
pub mod meteora_dlmm_swap;
pub mod meteora_dlmm_swap2;
pub mod pump_swap_buy;
pub mod pump_swap_sell;
pub mod raydium_amm_v4;
pub mod raydium_clmm;
pub mod raydium_cpmm;
pub mod whirlpool;

/// One pool's accounts, laid out the way its venue's swap instruction takes them, with the count
/// the menu entry declares. Only the venue modules construct one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VenueWindow {
    hop_kind: HopKind,
    account_count: u8,
    metas: Vec<AccountMeta>,
}

impl VenueWindow {
    pub(crate) fn new(hop_kind: HopKind, account_count: u8, metas: Vec<AccountMeta>) -> Self {
        debug_assert_eq!(usize::from(account_count), metas.len());
        Self {
            hop_kind,
            account_count,
            metas,
        }
    }

    /// The venue this window is for.
    #[must_use]
    pub const fn hop_kind(&self) -> HopKind {
        self.hop_kind
    }

    /// What the menu entry declares: the number of accounts the window carries, slot 0 included.
    #[must_use]
    pub const fn account_count(&self) -> u8 {
        self.account_count
    }

    /// The accounts, slot 0 first.
    #[must_use]
    pub fn account_metas(&self) -> &[AccountMeta] {
        &self.metas
    }

    /// Appends the accounts, slot 0 first, to an instruction's account list.
    pub fn append_account_metas(&self, out: &mut Vec<AccountMeta>) {
        out.extend_from_slice(&self.metas);
    }

    pub(crate) fn menu_entry(&self) -> [u8; MENU_ENTRY_LEN] {
        [self.hop_kind.discriminant(), self.account_count, 0, 0]
    }
}

pub(crate) fn readonly(key: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(key, false)
}

pub(crate) fn writable(key: Pubkey) -> AccountMeta {
    AccountMeta::new(key, false)
}

pub(crate) fn signer(key: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(key, true)
}

/// A venue's variable tail: at least one account and at most `MAX`, in the order the venue's
/// instruction reads them. The length is fixed at construction, so a window built from it
/// declares exactly the accounts it appends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PubkeyTail<const MAX: usize> {
    len: usize,
    keys: [Pubkey; MAX],
}

impl<const MAX: usize> PubkeyTail<MAX> {
    /// A tail of one account, the shortest any venue accepts.
    #[must_use]
    pub fn new(first: Pubkey) -> Self {
        let mut keys = [Pubkey::default(); MAX];
        if let Some(slot) = keys.first_mut() {
            *slot = first;
        }
        Self { len: 1, keys }
    }

    /// A tail of `1..=MAX` accounts, in order.
    ///
    /// # Errors
    ///
    /// [`Error::TailLength`] when the slice is empty or longer than `MAX`.
    pub fn try_from_slice(keys: &[Pubkey]) -> Result<Self, Error> {
        if keys.is_empty() || keys.len() > MAX {
            return Err(Error::TailLength {
                given: keys.len(),
                min: 1,
                max: MAX,
            });
        }
        let mut tail = [Pubkey::default(); MAX];
        for (slot, key) in tail.iter_mut().zip(keys) {
            *slot = *key;
        }
        Ok(Self {
            len: keys.len(),
            keys: tail,
        })
    }

    /// Appends one more account.
    ///
    /// # Errors
    ///
    /// [`Error::TailLength`] when the tail already holds `MAX` accounts.
    pub fn push(&mut self, key: Pubkey) -> Result<(), Error> {
        match self.keys.get_mut(self.len) {
            Some(slot) => {
                *slot = key;
                self.len = self.len.saturating_add(1);
                Ok(())
            }
            None => Err(Error::TailLength {
                given: self.len.saturating_add(1),
                min: 1,
                max: MAX,
            }),
        }
    }

    /// How many accounts the tail holds, `1..=MAX`.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Always false: a tail holds at least one account.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The accounts, in order.
    #[must_use]
    pub fn keys(&self) -> &[Pubkey] {
        self.keys.get(..self.len).unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    #[test]
    fn a_tail_holds_one_to_max_accounts() {
        assert_eq!(PubkeyTail::<3>::new(key(1)).len(), 1);
        assert_eq!(
            PubkeyTail::<3>::try_from_slice(&[key(1), key(2), key(3)])
                .unwrap()
                .keys(),
            &[key(1), key(2), key(3)]
        );
        assert_eq!(
            PubkeyTail::<3>::try_from_slice(&[]),
            Err(Error::TailLength {
                given: 0,
                min: 1,
                max: 3
            })
        );
        assert_eq!(
            PubkeyTail::<3>::try_from_slice(&[key(1); 4]),
            Err(Error::TailLength {
                given: 4,
                min: 1,
                max: 3
            })
        );
    }

    #[test]
    fn push_stops_at_max() {
        let mut tail = PubkeyTail::<2>::new(key(1));
        assert_eq!(tail.push(key(2)), Ok(()));
        assert_eq!(
            tail.push(key(3)),
            Err(Error::TailLength {
                given: 3,
                min: 1,
                max: 2
            })
        );
        assert_eq!(tail.keys(), &[key(1), key(2)]);
    }
}
