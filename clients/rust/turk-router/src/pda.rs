use solana_pubkey::Pubkey;

use crate::programs::ASSOCIATED_TOKEN_PROGRAM_ID;
use crate::wire::{CONFIG_SEED, ROUTER_PROGRAM_ID};

/// The router's config account: `find_program_address([CONFIG_SEED], ROUTER_PROGRAM_ID)`.
pub(crate) fn config_account() -> Pubkey {
    Pubkey::find_program_address(&[CONFIG_SEED], &ROUTER_PROGRAM_ID).0
}

pub(crate) fn associated_token_address(
    wallet: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    .0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::programs::{TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID};

    #[test]
    fn a_wallets_token_account_differs_per_token_program() {
        let wallet = Pubkey::new_from_array([7; 32]);
        let mint = Pubkey::new_from_array([9; 32]);
        assert_ne!(
            associated_token_address(&wallet, &mint, &TOKEN_PROGRAM_ID),
            associated_token_address(&wallet, &mint, &TOKEN_2022_PROGRAM_ID)
        );
    }
}
