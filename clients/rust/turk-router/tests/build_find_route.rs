//! The builder's local checks, both sides of every boundary, and the bytes and account list it
//! emits for an instruction the program would accept.

use solana_pubkey::Pubkey;
use turk_router::venues::raydium_amm_v4::{self, RaydiumAmmV4Accounts};
use turk_router::venues::raydium_cpmm::{self, RaydiumCpmmAccounts};
use turk_router::wire::{
    FIND_ROUTE_DISC, HEADER_LEN, MAX_MENU_ACCOUNTS, MAX_MENU_POOLS, MAX_ROUTE_MINTS,
    MENU_ENTRY_LEN, ROUTER_PROGRAM_ID,
};
use turk_router::{
    build_find_route_instruction, BaseMint, Error, FindRouteFlags, FindRouteParams, RouteMint,
    VenueWindow,
};

fn key(byte: u8) -> Pubkey {
    Pubkey::new_from_array([byte; 32])
}

fn amm_v4(seed: u8) -> VenueWindow {
    raydium_amm_v4::resolve(RaydiumAmmV4Accounts {
        pool: key(seed),
        base_vault: key(seed.wrapping_add(1)),
        quote_vault: key(seed.wrapping_add(2)),
        user_source: key(seed.wrapping_add(3)),
        user_destination: key(seed.wrapping_add(4)),
        payer: key(200),
    })
}

fn cpmm(seed: u8) -> VenueWindow {
    raydium_cpmm::resolve(RaydiumCpmmAccounts {
        user: key(200),
        amm_config: key(seed),
        pool: key(seed.wrapping_add(1)),
        input_token_account: key(seed.wrapping_add(2)),
        output_token_account: key(seed.wrapping_add(3)),
        input_vault: key(seed.wrapping_add(4)),
        output_vault: key(seed.wrapping_add(5)),
        input_token_program: turk_router::programs::TOKEN_PROGRAM_ID,
        output_token_program: turk_router::programs::TOKEN_PROGRAM_ID,
        input_mint: key(seed.wrapping_add(6)),
        output_mint: key(seed.wrapping_add(7)),
        observation_state: key(seed.wrapping_add(8)),
    })
}

fn mints(count: usize) -> Vec<RouteMint> {
    (0..count)
        .map(|index| RouteMint {
            token_program: turk_router::programs::TOKEN_PROGRAM_ID,
            user_ata: key(100 + u8::try_from(index).unwrap()),
        })
        .collect()
}

fn params<'a>(route_mints: &'a [RouteMint], menu: &'a [VenueWindow]) -> FindRouteParams<'a> {
    FindRouteParams {
        user: key(200),
        base_mint: BaseMint::Wsol,
        base_ata: key(201),
        fee_wallet: key(202),
        flags: FindRouteFlags {
            flashloan: false,
            fail_if_no_profit: true,
        },
        max_walk_steps: 7,
        min_profit_base_units: 0x0102_0304_0506_0708,
        route_mints,
        menu,
    }
}

#[test]
fn the_route_mint_count_is_bounded_on_both_sides() {
    let menu = [amm_v4(1)];
    assert_eq!(
        build_find_route_instruction(&params(&[], &menu)).err(),
        Some(Error::NoRouteMints)
    );
    assert!(build_find_route_instruction(&params(&mints(1), &menu)).is_ok());
    assert!(build_find_route_instruction(&params(&mints(MAX_ROUTE_MINTS), &menu)).is_ok());
    assert_eq!(
        build_find_route_instruction(&params(&mints(MAX_ROUTE_MINTS + 1), &menu)).err(),
        Some(Error::TooManyRouteMints {
            given: MAX_ROUTE_MINTS + 1,
            max: MAX_ROUTE_MINTS,
        })
    );
}

#[test]
fn the_menu_pool_count_is_bounded_on_both_sides() {
    let route = mints(1);
    assert_eq!(
        build_find_route_instruction(&params(&route, &[])).err(),
        Some(Error::EmptyMenu)
    );
    let one = [amm_v4(1)];
    assert!(build_find_route_instruction(&params(&route, &one)).is_ok());

    // Eight nine-account windows are seventy-two declared accounts: the pool count is legal and
    // the budget is what refuses them.
    let eight: Vec<VenueWindow> = (0..MAX_MENU_POOLS)
        .map(|i| amm_v4(u8::try_from(i).unwrap()))
        .collect();
    assert_eq!(
        build_find_route_instruction(&params(&route, &eight)).err(),
        Some(Error::MenuAccountBudgetExceeded {
            declared: 72,
            budget: MAX_MENU_ACCOUNTS,
        })
    );
    let nine: Vec<VenueWindow> = (0..=MAX_MENU_POOLS)
        .map(|i| amm_v4(u8::try_from(i).unwrap()))
        .collect();
    assert_eq!(
        build_find_route_instruction(&params(&route, &nine)).err(),
        Some(Error::TooManyMenuPools {
            given: MAX_MENU_POOLS + 1,
            max: MAX_MENU_POOLS,
        })
    );
}

#[test]
fn the_account_budget_admits_sixty_nine_and_refuses_seventy() {
    let route = mints(1);
    let exact = [
        cpmm(1),
        cpmm(20),
        cpmm(40),
        amm_v4(60),
        amm_v4(70),
        amm_v4(80),
    ];
    assert_eq!(
        exact
            .iter()
            .map(|w| usize::from(w.account_count()))
            .sum::<usize>(),
        MAX_MENU_ACCOUNTS
    );
    assert!(build_find_route_instruction(&params(&route, &exact)).is_ok());

    let over = [cpmm(1), cpmm(20), cpmm(40), cpmm(60), cpmm(80)];
    assert_eq!(
        build_find_route_instruction(&params(&route, &over)).err(),
        Some(Error::MenuAccountBudgetExceeded {
            declared: 70,
            budget: MAX_MENU_ACCOUNTS,
        })
    );
}

#[test]
fn the_data_is_the_header_then_one_entry_per_pool() {
    let route = mints(2);
    let menu = [amm_v4(1), cpmm(20)];
    let instruction = build_find_route_instruction(&params(&route, &menu)).unwrap();

    assert_eq!(instruction.program_id, ROUTER_PROGRAM_ID);
    assert_eq!(
        instruction.data.len(),
        HEADER_LEN + MENU_ENTRY_LEN * menu.len()
    );
    assert_eq!(&instruction.data[..8], &FIND_ROUTE_DISC);
    assert_eq!(instruction.data[8], 0b10, "flags: fail_if_no_profit only");
    assert_eq!(
        instruction.data[9], 7,
        "max_walk_steps passes through as given"
    );
    assert_eq!(instruction.data[10], 2, "num_mints");
    assert_eq!(instruction.data[11], 2, "num_pools");
    assert_eq!(&instruction.data[12..20], &[8, 7, 6, 5, 4, 3, 2, 1]);
    assert_eq!(&instruction.data[20..24], &[0, 9, 0, 0]);
    assert_eq!(&instruction.data[24..28], &[3, 14, 0, 0]);
}

#[test]
fn both_flag_bits_and_neither_are_representable_and_nothing_else() {
    for (flags, byte) in [
        (FindRouteFlags::default(), 0u8),
        (
            FindRouteFlags {
                flashloan: true,
                fail_if_no_profit: false,
            },
            1,
        ),
        (
            FindRouteFlags {
                flashloan: false,
                fail_if_no_profit: true,
            },
            2,
        ),
        (
            FindRouteFlags {
                flashloan: true,
                fail_if_no_profit: true,
            },
            3,
        ),
    ] {
        assert_eq!(flags.to_byte(), byte);
    }
}

#[test]
fn the_account_list_is_prefix_then_route_mints_then_windows_in_order() {
    let route = mints(2);
    let menu = [amm_v4(1), cpmm(20)];
    let params = params(&route, &menu);
    let instruction = build_find_route_instruction(&params).unwrap();
    let accounts = &instruction.accounts;

    let base_mint = BaseMint::Wsol.mint();
    assert_eq!(
        (
            accounts[0].pubkey,
            accounts[0].is_signer,
            accounts[0].is_writable
        ),
        (params.user, true, false)
    );
    assert_eq!(
        (
            accounts[1].pubkey,
            accounts[1].is_signer,
            accounts[1].is_writable
        ),
        (params.base_ata, false, true)
    );
    assert_eq!(
        (
            accounts[2].pubkey,
            accounts[2].is_signer,
            accounts[2].is_writable
        ),
        (base_mint, false, false)
    );
    assert_eq!(
        (
            accounts[3].pubkey,
            accounts[3].is_signer,
            accounts[3].is_writable
        ),
        (turk_router::programs::TOKEN_PROGRAM_ID, false, false)
    );
    let config =
        Pubkey::find_program_address(&[turk_router::wire::CONFIG_SEED], &ROUTER_PROGRAM_ID).0;
    assert_eq!(
        (
            accounts[4].pubkey,
            accounts[4].is_signer,
            accounts[4].is_writable
        ),
        (config, false, false)
    );
    let fee_ata = Pubkey::find_program_address(
        &[
            params.fee_wallet.as_ref(),
            turk_router::programs::TOKEN_PROGRAM_ID.as_ref(),
            base_mint.as_ref(),
        ],
        &turk_router::programs::ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        (
            accounts[5].pubkey,
            accounts[5].is_signer,
            accounts[5].is_writable
        ),
        (fee_ata, false, true)
    );

    for (index, mint) in route.iter().enumerate() {
        let program = &accounts[6 + 2 * index];
        let ata = &accounts[7 + 2 * index];
        assert_eq!(
            (program.pubkey, program.is_writable),
            (mint.token_program, false)
        );
        assert_eq!((ata.pubkey, ata.is_writable), (mint.user_ata, true));
    }

    let mut expected = Vec::new();
    for window in &menu {
        window.append_account_metas(&mut expected);
    }
    assert_eq!(&accounts[6 + 2 * route.len()..], expected.as_slice());
}

#[test]
fn the_fee_token_account_follows_the_base_mint() {
    let route = mints(1);
    let menu = [amm_v4(1)];
    let mut wsol = params(&route, &menu);
    let mut usdc = params(&route, &menu);
    wsol.base_mint = BaseMint::Wsol;
    usdc.base_mint = BaseMint::Usdc;
    let wsol_ix = build_find_route_instruction(&wsol).unwrap();
    let usdc_ix = build_find_route_instruction(&usdc).unwrap();
    assert_eq!(wsol_ix.accounts[2].pubkey, BaseMint::Wsol.mint());
    assert_eq!(usdc_ix.accounts[2].pubkey, BaseMint::Usdc.mint());
    assert_ne!(wsol_ix.accounts[5].pubkey, usdc_ix.accounts[5].pubkey);
}
