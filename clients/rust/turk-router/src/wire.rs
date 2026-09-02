//! The `find_route` wire: every number this crate encodes, in one place.
//!
//! The instruction data is fixed-width: a 20-byte header, then one 4-byte menu entry per pool,
//! and nothing else — the program refuses data of any other length, padded or truncated. The
//! header is the discriminator, the flags byte, `max_walk_steps`, `num_mints`, `num_pools`, and
//! `min_profit_base_units` as a little-endian `u64`. A menu entry is `hop_kind`, `account_count`,
//! and the two transfer-hook group lengths, which this crate always sends as zero.
//!
//! Each constant here is held against `wire/wire-manifest.json` by the test suite. A wire literal
//! anywhere else in the crate is a defect.

use solana_pubkey::Pubkey;

/// `TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am` — the router program.
pub const ROUTER_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    6, 200, 26, 170, 58, 47, 218, 193, 91, 166, 109, 28, 87, 105, 91, 122, 172, 249, 66, 243, 185,
    234, 214, 247, 216, 205, 127, 253, 82, 0, 152, 38,
]);

/// The one seed of the router's config account.
pub const CONFIG_SEED: &[u8] = b"config";

/// The instruction discriminator: `sha256("global:find_route")[..8]`.
pub const FIND_ROUTE_DISC: [u8; 8] = [0x63, 0x61, 0x70, 0x5d, 0x8f, 0x05, 0x5e, 0x00];

/// Bytes before the first menu entry.
pub const HEADER_LEN: usize = 20;

/// Bytes per menu entry.
pub const MENU_ENTRY_LEN: usize = 4;

/// The most pools a menu may name.
pub const MAX_MENU_POOLS: usize = 8;

/// The most route mints an instruction may name.
pub const MAX_ROUTE_MINTS: usize = 4;

/// The ceiling on every window's `account_count`, summed over the menu.
pub const MAX_MENU_ACCOUNTS: usize = 69;

/// The ceiling on one transfer-hook account group. This crate declares no groups; the number is
/// carried so the manifest agreement is complete.
pub const MAX_HOOK_GROUP_LEN: usize = 21;

/// Flags bit 0: the base token account holds borrowed principal a later instruction repays. The
/// program then behaves as if [`FLAG_FAIL_IF_NO_PROFIT`] were set.
pub const FLAG_FLASHLOAN: u8 = 1 << 0;

/// Flags bit 1: fail the instruction, rather than settle a loss, when no cycle clears the profit
/// threshold.
pub const FLAG_FAIL_IF_NO_PROFIT: u8 = 1 << 1;

/// The six accounts before the route-mint section, with the flags a caller sends:
/// `(name, is_signer, is_writable)`.
pub const PREFIX_ACCOUNT_METAS: [(&str, bool, bool); 6] = [
    ("user", true, false),
    ("base_ata", false, true),
    ("base_mint", false, false),
    ("base_token_program", false, false),
    ("config_account", false, false),
    ("fee_ata", false, true),
];

/// The header fields a caller chooses. Counts are supplied by the builder, which has already
/// bounded them.
pub(crate) struct Header {
    pub(crate) flags: u8,
    pub(crate) max_walk_steps: u8,
    pub(crate) num_mints: u8,
    pub(crate) num_pools: u8,
    pub(crate) min_profit_base_units: u64,
}

/// Lays out the header and the entries; exactly `HEADER_LEN + MENU_ENTRY_LEN * entries.len()`
/// bytes.
pub(crate) fn encode(header: &Header, entries: &[[u8; MENU_ENTRY_LEN]]) -> Vec<u8> {
    let mut data =
        Vec::with_capacity(HEADER_LEN.saturating_add(entries.len().saturating_mul(MENU_ENTRY_LEN)));
    data.extend_from_slice(&FIND_ROUTE_DISC);
    data.push(header.flags);
    data.push(header.max_walk_steps);
    data.push(header.num_mints);
    data.push(header.num_pools);
    data.extend_from_slice(&header.min_profit_base_units.to_le_bytes());
    for entry in entries {
        data.extend_from_slice(entry);
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_program_id_is_the_documented_address() {
        assert_eq!(
            ROUTER_PROGRAM_ID.to_string(),
            "TURKbNaes5RA3sMnkRsCmuPBKZbPTyRxv9cTiMQ43Am"
        );
    }

    #[test]
    fn the_header_is_twenty_bytes_in_field_order() {
        let header = Header {
            flags: FLAG_FAIL_IF_NO_PROFIT,
            max_walk_steps: 3,
            num_mints: 2,
            num_pools: 1,
            min_profit_base_units: 0x0102_0304_0506_0708,
        };
        let data = encode(&header, &[[0, 9, 0, 0]]);
        assert_eq!(data.len(), HEADER_LEN + MENU_ENTRY_LEN);
        assert_eq!(&data[..8], &FIND_ROUTE_DISC);
        assert_eq!(&data[8..12], &[2, 3, 2, 1]);
        assert_eq!(&data[12..20], &[8, 7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(&data[20..], &[0, 9, 0, 0]);
    }
}
