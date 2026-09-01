//! Program and sysvar addresses that several venue windows name in fixed positions.

use solana_pubkey::Pubkey;

/// `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` — the Token program.
pub const TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 28, 180, 133, 237,
    95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169,
]);

/// `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` — the Token Extensions program.
pub const TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    6, 221, 246, 225, 238, 117, 143, 222, 24, 66, 93, 188, 228, 108, 205, 218, 182, 26, 252, 77,
    131, 185, 13, 39, 254, 189, 249, 40, 216, 161, 139, 252,
]);

/// `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` — the Associated Token Account program.
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    140, 151, 37, 143, 78, 36, 137, 241, 187, 61, 16, 41, 20, 142, 13, 131, 11, 90, 19, 153, 218,
    255, 16, 132, 4, 142, 123, 216, 219, 233, 248, 89,
]);

/// `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` — the Memo program.
pub const MEMO_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    5, 74, 83, 90, 153, 41, 33, 6, 77, 36, 232, 113, 96, 218, 56, 124, 124, 53, 181, 221, 188, 146,
    187, 129, 228, 31, 168, 64, 65, 5, 68, 141,
]);

/// `Sysvar1nstructions1111111111111111111111111` — the instructions sysvar.
pub const INSTRUCTIONS_SYSVAR_ID: Pubkey = Pubkey::new_from_array([
    6, 167, 213, 23, 24, 123, 209, 102, 53, 218, 212, 4, 85, 253, 194, 192, 193, 36, 198, 143, 33,
    86, 117, 165, 219, 186, 203, 95, 8, 0, 0, 0,
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_addresses_are_the_documented_ones() {
        for (address, name) in [
            (
                TOKEN_PROGRAM_ID,
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            ),
            (
                TOKEN_2022_PROGRAM_ID,
                "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
            ),
            (
                ASSOCIATED_TOKEN_PROGRAM_ID,
                "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
            ),
            (
                MEMO_PROGRAM_ID,
                "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
            ),
            (
                INSTRUCTIONS_SYSVAR_ID,
                "Sysvar1nstructions1111111111111111111111111",
            ),
        ] {
            assert_eq!(address.to_string(), name);
        }
    }
}
