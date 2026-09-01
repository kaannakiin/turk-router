use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;

use crate::pda::{associated_token_address, config_account};
use crate::programs::TOKEN_PROGRAM_ID;
use crate::venues::VenueWindow;
use crate::wire::{
    encode, Header, FLAG_FAIL_IF_NO_PROFIT, FLAG_FLASHLOAN, MAX_MENU_ACCOUNTS, MAX_MENU_POOLS,
    MAX_ROUTE_MINTS, ROUTER_PROGRAM_ID,
};
use crate::Error;

/// The two mints a cycle may start and end on. The program accepts no other base mint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BaseMint {
    /// `So11111111111111111111111111111111111111112`, wrapped SOL.
    Wsol,
    /// `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`, USDC.
    Usdc,
}

impl BaseMint {
    /// Both, in the program's order.
    pub const ALL: [BaseMint; 2] = [BaseMint::Wsol, BaseMint::Usdc];

    /// The mint address.
    #[must_use]
    pub const fn mint(self) -> Pubkey {
        match self {
            BaseMint::Wsol => Pubkey::new_from_array([
                6, 155, 136, 87, 254, 171, 129, 132, 251, 104, 127, 99, 70, 24, 192, 53, 218, 196,
                57, 220, 26, 235, 59, 85, 152, 160, 240, 0, 0, 0, 0, 1,
            ]),
            BaseMint::Usdc => Pubkey::new_from_array([
                198, 250, 122, 243, 190, 219, 173, 58, 61, 101, 243, 106, 171, 201, 116, 49, 177,
                187, 228, 194, 210, 246, 224, 228, 124, 166, 2, 3, 69, 47, 93, 97,
            ]),
        }
    }
}

/// The flags byte, as the two bits it defines. The other six bits are refused by the program and
/// cannot be set from here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FindRouteFlags {
    /// The base token account holds borrowed principal a later instruction in the same
    /// transaction repays. The program then also fails rather than settle a loss.
    pub flashloan: bool,
    /// Fail the instruction, rather than settle a loss, when no cycle clears the profit
    /// threshold.
    pub fail_if_no_profit: bool,
}

impl FindRouteFlags {
    /// The byte the header carries.
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        let mut byte = 0;
        if self.flashloan {
            byte |= FLAG_FLASHLOAN;
        }
        if self.fail_if_no_profit {
            byte |= FLAG_FAIL_IF_NO_PROFIT;
        }
        byte
    }
}

/// One mint the route may pass through, with the user's token account for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteMint {
    /// The program that owns `user_ata`: the Token program or the Token Extensions program.
    pub token_program: Pubkey,
    /// The user's token account for the mint. The program requires `token_program` to own it.
    pub user_ata: Pubkey,
}

/// Everything one `find_route` instruction needs.
///
/// # Route mints are the route graph
///
/// The base mint is node 0, and the `i`-th entry of `route_mints` is node `i + 1`. The program
/// numbers the graph it searches by this order, so reordering the entries changes which cycles
/// exist, not merely the account list. A route mint equal to the base mint, or repeated, is
/// refused by the program.
#[derive(Clone, Debug)]
pub struct FindRouteParams<'a> {
    /// The signer whose token accounts the route moves through.
    pub user: Pubkey,
    /// Which of the two base mints the cycle starts and ends on.
    pub base_mint: BaseMint,
    /// The user's token account for the base mint. Supplied, never derived: the program only
    /// requires it to be owned by the Token program and hold the base mint.
    pub base_ata: Pubkey,
    /// The router's fee collector, as the wallet address its config stores. The builder derives
    /// that wallet's token account for `base_mint`; the program reads it only at a nonzero fee
    /// rate, but the slot is sent on every call.
    pub fee_wallet: Pubkey,
    /// The flags byte.
    pub flags: FindRouteFlags,
    /// How many steps a walk-venue quote may take. Sent as given: the program substitutes its
    /// default for zero and clamps a value above its cap.
    pub max_walk_steps: u8,
    /// The least profit, in the base mint's minor units and net of the router's fee, a cycle must
    /// clear to be executed. The program treats zero as one.
    pub min_profit_base_units: u64,
    /// The mints the route may pass through, in node order. `1..=MAX_ROUTE_MINTS`.
    pub route_mints: &'a [RouteMint],
    /// The pools the program may choose among, in the order they are declared. `1..=MAX_MENU_POOLS`
    /// windows whose account counts sum to at most `MAX_MENU_ACCOUNTS`.
    pub menu: &'a [VenueWindow],
}

/// Builds the instruction.
///
/// # Errors
///
/// [`Error::NoRouteMints`] or [`Error::TooManyRouteMints`] for a route mint list outside
/// `1..=MAX_ROUTE_MINTS`; [`Error::EmptyMenu`] or [`Error::TooManyMenuPools`] for a menu outside
/// `1..=MAX_MENU_POOLS`; [`Error::MenuAccountBudgetExceeded`] when the windows' account counts
/// sum past `MAX_MENU_ACCOUNTS`. The program would refuse each of these as malformed instruction
/// data before reaching any error of its own.
///
/// What this crate cannot check is left to the program: that `user` signs, that the token
/// accounts are owned as declared, that the fee collector's token account exists when the fee
/// rate is nonzero, that the router is not paused, and that a profitable cycle exists.
pub fn build_find_route_instruction(params: &FindRouteParams<'_>) -> Result<Instruction, Error> {
    let num_mints = bounded_count(
        params.route_mints.len(),
        MAX_ROUTE_MINTS,
        Error::NoRouteMints,
        |given| Error::TooManyRouteMints {
            given,
            max: MAX_ROUTE_MINTS,
        },
    )?;
    let num_pools = bounded_count(
        params.menu.len(),
        MAX_MENU_POOLS,
        Error::EmptyMenu,
        |given| Error::TooManyMenuPools {
            given,
            max: MAX_MENU_POOLS,
        },
    )?;

    let declared = params
        .menu
        .iter()
        .try_fold(0usize, |total, window| {
            total.checked_add(usize::from(window.account_count()))
        })
        .unwrap_or(usize::MAX);
    if declared > MAX_MENU_ACCOUNTS {
        return Err(Error::MenuAccountBudgetExceeded {
            declared,
            budget: MAX_MENU_ACCOUNTS,
        });
    }

    let base_mint = params.base_mint.mint();
    let fee_ata = associated_token_address(&params.fee_wallet, &base_mint, &TOKEN_PROGRAM_ID);

    let mut accounts = vec![
        AccountMeta::new_readonly(params.user, true),
        AccountMeta::new(params.base_ata, false),
        AccountMeta::new_readonly(base_mint, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(config_account(), false),
        AccountMeta::new(fee_ata, false),
    ];
    for mint in params.route_mints {
        accounts.push(AccountMeta::new_readonly(mint.token_program, false));
        accounts.push(AccountMeta::new(mint.user_ata, false));
    }
    for window in params.menu {
        window.append_account_metas(&mut accounts);
    }

    let entries: Vec<[u8; 4]> = params.menu.iter().map(VenueWindow::menu_entry).collect();
    let data = encode(
        &Header {
            flags: params.flags.to_byte(),
            max_walk_steps: params.max_walk_steps,
            num_mints,
            num_pools,
            min_profit_base_units: params.min_profit_base_units,
        },
        &entries,
    );

    Ok(Instruction {
        program_id: ROUTER_PROGRAM_ID,
        accounts,
        data,
    })
}

fn bounded_count(
    given: usize,
    max: usize,
    empty: Error,
    over: impl FnOnce(usize) -> Error,
) -> Result<u8, Error> {
    if given == 0 {
        return Err(empty);
    }
    if given > max {
        return Err(over(given));
    }
    u8::try_from(given).map_err(|_| over(given))
}
