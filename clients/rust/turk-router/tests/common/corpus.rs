//! The cross-language golden corpus: typed inputs to `build_find_route_instruction`, the bytes
//! and account list this crate emits for each, and the sweep that generates them.
//!
//! Only this crate writes `clients/golden/find_route.json`; the TypeScript client reads the same
//! file and asserts the same outputs. Addresses are base58, bytes are hex and the `u64` is a
//! decimal string, so the file carries no numeric array the publish gate could mistake for a key.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use turk_router::programs::{TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID};
use turk_router::venues::meteora_damm_v1::{self, MeteoraDammV1Accounts};
use turk_router::venues::meteora_damm_v2::{self, DammV2Form, MeteoraDammV2Accounts};
use turk_router::venues::meteora_dlmm_swap::{self, BinArrayTail, MeteoraDlmmSwapAccounts};
use turk_router::venues::meteora_dlmm_swap2::{self, MeteoraDlmmSwap2Accounts, MAX_BIN_ARRAYS};
use turk_router::venues::pump_swap_buy::{self, PumpSwapBuyAccounts};
use turk_router::venues::pump_swap_sell::{self, PumpSwapSellAccounts};
use turk_router::venues::raydium_amm_v4::{self, RaydiumAmmV4Accounts};
use turk_router::venues::raydium_clmm::{self, ClmmTail, RaydiumClmmAccounts};
use turk_router::venues::raydium_cpmm::{self, RaydiumCpmmAccounts};
use turk_router::venues::whirlpool::{self, SupplementalTickArrays, WhirlpoolAccounts};
use turk_router::venues::PubkeyTail;
use turk_router::{
    build_find_route_instruction, BaseMint, Error, FindRouteFlags, FindRouteParams, RouteMint,
    VenueWindow,
};

pub const GENERATOR: &str = "clients/rust/turk-router/tests/cross_language.rs";
pub const REGENERATE: &str =
    "TURK_ROUTER_WRITE_GOLDEN=1 cargo test -p turk-router --test cross_language";

/// The first seventeen bytes of every synthetic address; the index sits in the last eight.
const KEY_TAG: &[u8; 17] = b"turk-router-xlang";

pub type Addr = String;

pub fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden/find_route.json")
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corpus {
    pub cases: BTreeMap<String, Case>,
    pub generator: String,
    pub regenerate: String,
    pub wire_epoch: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub params: CaseParams,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<Expected>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseParams {
    pub base_ata: Addr,
    pub base_mint: Addr,
    pub fee_wallet: Addr,
    pub flags: FlagsJson,
    pub max_walk_steps: u8,
    pub menu: Vec<WindowInput>,
    pub min_profit_base_units: String,
    pub route_mints: Vec<RouteMintJson>,
    pub user: Addr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlagsJson {
    pub fail_if_no_profit: bool,
    pub flashloan: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteMintJson {
    pub token_program: Addr,
    pub user_ata: Addr,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    pub accounts: Vec<String>,
    pub data_hex: String,
    pub program_address: Addr,
}

/// One menu window as the caller names it: the venue's own account fields, plus its variable
/// tail or form. The tag is the manifest's kind name.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum WindowInput {
    RaydiumAmmV4 {
        accounts: RaydiumAmmV4Json,
    },
    Whirlpool {
        accounts: WhirlpoolJson,
        supplemental_tick_arrays: Vec<Addr>,
    },
    RaydiumClmm {
        accounts: RaydiumClmmJson,
        tail: Vec<Addr>,
    },
    RaydiumCpmm {
        accounts: RaydiumCpmmJson,
    },
    MeteoraDlmmSwap {
        accounts: MeteoraDlmmSwapJson,
        bin_arrays: Vec<Addr>,
    },
    MeteoraDlmmSwap2 {
        accounts: MeteoraDlmmSwap2Json,
        bin_arrays: Vec<Addr>,
    },
    MeteoraDammV2 {
        accounts: MeteoraDammV2Json,
        form: DammV2FormJson,
    },
    PumpSwapSell {
        accounts: PumpSwapSellJson,
    },
    PumpSwapBuy {
        accounts: PumpSwapBuyJson,
    },
    MeteoraDammV1 {
        accounts: MeteoraDammV1Json,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DammV2FormJson {
    Base,
    RateLimited,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaydiumAmmV4Json {
    pub pool: Addr,
    pub base_vault: Addr,
    pub quote_vault: Addr,
    pub user_source: Addr,
    pub user_destination: Addr,
    pub payer: Addr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WhirlpoolJson {
    pub token_program_a: Addr,
    pub token_program_b: Addr,
    pub token_authority: Addr,
    pub whirlpool: Addr,
    pub mint_a: Addr,
    pub mint_b: Addr,
    pub token_owner_account_a: Addr,
    pub token_vault_a: Addr,
    pub token_owner_account_b: Addr,
    pub token_vault_b: Addr,
    pub tick_array_0: Addr,
    pub tick_array_1: Addr,
    pub tick_array_2: Addr,
    pub oracle: Addr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaydiumClmmJson {
    pub payer: Addr,
    pub amm_config: Addr,
    pub pool: Addr,
    pub input_token_account: Addr,
    pub output_token_account: Addr,
    pub input_vault: Addr,
    pub output_vault: Addr,
    pub observation_state: Addr,
    pub input_mint: Addr,
    pub output_mint: Addr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RaydiumCpmmJson {
    pub user: Addr,
    pub amm_config: Addr,
    pub pool: Addr,
    pub input_token_account: Addr,
    pub output_token_account: Addr,
    pub input_vault: Addr,
    pub output_vault: Addr,
    pub input_token_program: Addr,
    pub output_token_program: Addr,
    pub input_mint: Addr,
    pub output_mint: Addr,
    pub observation_state: Addr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeteoraDlmmSwapJson {
    pub lb_pair: Addr,
    pub bin_array_bitmap_extension: Option<Addr>,
    pub reserve_x: Addr,
    pub reserve_y: Addr,
    pub user_token_in: Addr,
    pub user_token_out: Addr,
    pub mint_x: Addr,
    pub mint_y: Addr,
    pub oracle: Addr,
    pub host_fee_in: Option<Addr>,
    pub user: Addr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeteoraDlmmSwap2Json {
    pub pool: Addr,
    pub bin_array_bitmap_extension: Option<Addr>,
    pub reserve_x: Addr,
    pub reserve_y: Addr,
    pub user_token_in: Addr,
    pub user_token_out: Addr,
    pub token_x_mint: Addr,
    pub token_y_mint: Addr,
    pub oracle: Addr,
    pub host_fee_in: Option<Addr>,
    pub user: Addr,
    pub token_x_program: Addr,
    pub token_y_program: Addr,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeteoraDammV2Json {
    pub pool: Addr,
    pub input_token_account: Addr,
    pub output_token_account: Addr,
    pub token_a_vault: Addr,
    pub token_b_vault: Addr,
    pub token_a_mint: Addr,
    pub token_b_mint: Addr,
    pub payer: Addr,
    pub token_a_program: Addr,
    pub token_b_program: Addr,
    pub referral_token_account: Option<Addr>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PumpSwapSellJson {
    pub pool: Addr,
    pub user: Addr,
    pub forwarded_before_base_mint: Addr,
    pub base_mint: Addr,
    pub quote_mint: Addr,
    pub base_ata: Addr,
    pub quote_ata: Addr,
    pub base_vault: Addr,
    pub quote_vault: Addr,
    pub forwarded_before_fee_config: [Addr; 10],
    pub cashback: Option<[Addr; 2]>,
    pub pool_v2: Option<Addr>,
    pub forwarded_close: [Addr; 2],
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PumpSwapBuyJson {
    pub pool: Addr,
    pub user: Addr,
    pub forwarded_before_base_mint: Addr,
    pub base_mint: Addr,
    pub quote_mint: Addr,
    pub base_token_account: Addr,
    pub quote_token_account: Addr,
    pub base_vault: Addr,
    pub quote_vault: Addr,
    pub forwarded_before_volume_accumulator: [Addr; 10],
    pub user_volume_accumulator: Addr,
    pub forwarded_close: [Addr; 2],
    pub pool_v2: Option<Addr>,
    pub cashback: Option<Addr>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeteoraDammV1Json {
    pub pool: Addr,
    pub user_source: Addr,
    pub user_dest: Addr,
    pub a_vault: Addr,
    pub b_vault: Addr,
    pub a_token_vault: Addr,
    pub b_token_vault: Addr,
    pub a_vault_lp_mint: Addr,
    pub b_vault_lp_mint: Addr,
    pub a_vault_lp: Addr,
    pub b_vault_lp: Addr,
    pub protocol_token_fee: Addr,
    pub payer: Addr,
}

// ---------------------------------------------------------------------------------------------
// Synthetic addresses
// ---------------------------------------------------------------------------------------------

/// A tagged address that is never a real key: the tag, zeros, then the index little-endian.
pub fn key(index: u64) -> Pubkey {
    let mut bytes = [0u8; 32];
    bytes[..KEY_TAG.len()].copy_from_slice(KEY_TAG);
    bytes[24..].copy_from_slice(&index.to_le_bytes());
    Pubkey::new_from_array(bytes)
}

fn addr(index: u64) -> Addr {
    key(index).to_string()
}

/// Per-case counter for window accounts. Every case starts it afresh, so adding or reordering a
/// case never renumbers another case's addresses.
struct Keys {
    next: u64,
}

impl Keys {
    fn new() -> Self {
        Self { next: 100 }
    }

    fn next(&mut self) -> Addr {
        let address = addr(self.next);
        self.next += 1;
        address
    }

    fn many(&mut self, count: usize) -> Vec<Addr> {
        (0..count).map(|_| self.next()).collect()
    }

    fn ten(&mut self) -> [Addr; 10] {
        std::array::from_fn(|_| self.next())
    }

    fn two(&mut self) -> [Addr; 2] {
        [self.next(), self.next()]
    }
}

const USER: u64 = 1;
const BASE_ATA: u64 = 2;
const FEE_WALLET: u64 = 3;
const FIRST_ROUTE_ATA: u64 = 10;

// ---------------------------------------------------------------------------------------------
// Window inputs, one constructor per kind and variant
// ---------------------------------------------------------------------------------------------

fn amm_v4(keys: &mut Keys) -> WindowInput {
    WindowInput::RaydiumAmmV4 {
        accounts: RaydiumAmmV4Json {
            pool: keys.next(),
            base_vault: keys.next(),
            quote_vault: keys.next(),
            user_source: keys.next(),
            user_destination: keys.next(),
            payer: addr(USER),
        },
    }
}

fn whirlpool_window(keys: &mut Keys, supplemental: usize) -> WindowInput {
    WindowInput::Whirlpool {
        accounts: WhirlpoolJson {
            token_program_a: TOKEN_PROGRAM_ID.to_string(),
            token_program_b: TOKEN_2022_PROGRAM_ID.to_string(),
            token_authority: addr(USER),
            whirlpool: keys.next(),
            mint_a: keys.next(),
            mint_b: keys.next(),
            token_owner_account_a: keys.next(),
            token_vault_a: keys.next(),
            token_owner_account_b: keys.next(),
            token_vault_b: keys.next(),
            tick_array_0: keys.next(),
            tick_array_1: keys.next(),
            tick_array_2: keys.next(),
            oracle: keys.next(),
        },
        supplemental_tick_arrays: keys.many(supplemental),
    }
}

fn clmm(keys: &mut Keys, tail: usize) -> WindowInput {
    WindowInput::RaydiumClmm {
        accounts: RaydiumClmmJson {
            payer: addr(USER),
            amm_config: keys.next(),
            pool: keys.next(),
            input_token_account: keys.next(),
            output_token_account: keys.next(),
            input_vault: keys.next(),
            output_vault: keys.next(),
            observation_state: keys.next(),
            input_mint: keys.next(),
            output_mint: keys.next(),
        },
        tail: keys.many(tail),
    }
}

fn cpmm(keys: &mut Keys) -> WindowInput {
    WindowInput::RaydiumCpmm {
        accounts: RaydiumCpmmJson {
            user: addr(USER),
            amm_config: keys.next(),
            pool: keys.next(),
            input_token_account: keys.next(),
            output_token_account: keys.next(),
            input_vault: keys.next(),
            output_vault: keys.next(),
            input_token_program: TOKEN_PROGRAM_ID.to_string(),
            output_token_program: TOKEN_2022_PROGRAM_ID.to_string(),
            input_mint: keys.next(),
            output_mint: keys.next(),
            observation_state: keys.next(),
        },
    }
}

fn dlmm_swap(keys: &mut Keys, bins: usize, bitmap: bool, host_fee: bool) -> WindowInput {
    WindowInput::MeteoraDlmmSwap {
        accounts: MeteoraDlmmSwapJson {
            lb_pair: keys.next(),
            bin_array_bitmap_extension: bitmap.then(|| keys.next()),
            reserve_x: keys.next(),
            reserve_y: keys.next(),
            user_token_in: keys.next(),
            user_token_out: keys.next(),
            mint_x: keys.next(),
            mint_y: keys.next(),
            oracle: keys.next(),
            host_fee_in: host_fee.then(|| keys.next()),
            user: addr(USER),
        },
        bin_arrays: keys.many(bins),
    }
}

fn dlmm_swap2(keys: &mut Keys, bins: usize, bitmap: bool, host_fee: bool) -> WindowInput {
    let token_y_program = if bins.is_multiple_of(2) {
        TOKEN_2022_PROGRAM_ID
    } else {
        TOKEN_PROGRAM_ID
    };
    WindowInput::MeteoraDlmmSwap2 {
        accounts: MeteoraDlmmSwap2Json {
            pool: keys.next(),
            bin_array_bitmap_extension: bitmap.then(|| keys.next()),
            reserve_x: keys.next(),
            reserve_y: keys.next(),
            user_token_in: keys.next(),
            user_token_out: keys.next(),
            token_x_mint: keys.next(),
            token_y_mint: keys.next(),
            oracle: keys.next(),
            host_fee_in: host_fee.then(|| keys.next()),
            user: addr(USER),
            token_x_program: TOKEN_PROGRAM_ID.to_string(),
            token_y_program: token_y_program.to_string(),
        },
        bin_arrays: keys.many(bins),
    }
}

fn damm_v2(keys: &mut Keys, form: DammV2FormJson, referral: bool) -> WindowInput {
    WindowInput::MeteoraDammV2 {
        accounts: MeteoraDammV2Json {
            pool: keys.next(),
            input_token_account: keys.next(),
            output_token_account: keys.next(),
            token_a_vault: keys.next(),
            token_b_vault: keys.next(),
            token_a_mint: keys.next(),
            token_b_mint: keys.next(),
            payer: addr(USER),
            token_a_program: TOKEN_PROGRAM_ID.to_string(),
            token_b_program: TOKEN_2022_PROGRAM_ID.to_string(),
            referral_token_account: referral.then(|| keys.next()),
        },
        form,
    }
}

fn pump_sell(keys: &mut Keys, cashback: bool, pool_v2: bool) -> WindowInput {
    WindowInput::PumpSwapSell {
        accounts: PumpSwapSellJson {
            pool: keys.next(),
            user: addr(USER),
            forwarded_before_base_mint: keys.next(),
            base_mint: keys.next(),
            quote_mint: keys.next(),
            base_ata: keys.next(),
            quote_ata: keys.next(),
            base_vault: keys.next(),
            quote_vault: keys.next(),
            forwarded_before_fee_config: keys.ten(),
            cashback: cashback.then(|| keys.two()),
            pool_v2: pool_v2.then(|| keys.next()),
            forwarded_close: keys.two(),
        },
    }
}

fn pump_buy(keys: &mut Keys, cashback: bool, pool_v2: bool) -> WindowInput {
    WindowInput::PumpSwapBuy {
        accounts: PumpSwapBuyJson {
            pool: keys.next(),
            user: addr(USER),
            forwarded_before_base_mint: keys.next(),
            base_mint: keys.next(),
            quote_mint: keys.next(),
            base_token_account: keys.next(),
            quote_token_account: keys.next(),
            base_vault: keys.next(),
            quote_vault: keys.next(),
            forwarded_before_volume_accumulator: keys.ten(),
            user_volume_accumulator: keys.next(),
            forwarded_close: keys.two(),
            pool_v2: pool_v2.then(|| keys.next()),
            cashback: cashback.then(|| keys.next()),
        },
    }
}

fn damm_v1(keys: &mut Keys) -> WindowInput {
    WindowInput::MeteoraDammV1 {
        accounts: MeteoraDammV1Json {
            pool: keys.next(),
            user_source: keys.next(),
            user_dest: keys.next(),
            a_vault: keys.next(),
            b_vault: keys.next(),
            a_token_vault: keys.next(),
            b_token_vault: keys.next(),
            a_vault_lp_mint: keys.next(),
            b_vault_lp_mint: keys.next(),
            a_vault_lp: keys.next(),
            b_vault_lp: keys.next(),
            protocol_token_fee: keys.next(),
            payer: addr(USER),
        },
    }
}

// ---------------------------------------------------------------------------------------------
// The sweep
// ---------------------------------------------------------------------------------------------

struct Header {
    mints: usize,
    flags: FlagsJson,
    max_walk_steps: u8,
    min_profit_base_units: u64,
    base_mint: BaseMint,
}

const PLAIN: Header = Header {
    mints: 1,
    flags: FlagsJson {
        fail_if_no_profit: false,
        flashloan: false,
    },
    max_walk_steps: 0,
    min_profit_base_units: 0,
    base_mint: BaseMint::Wsol,
};

fn params(header: &Header, menu: Vec<WindowInput>) -> CaseParams {
    let route_mints = (0..header.mints)
        .map(|index| RouteMintJson {
            token_program: if index.is_multiple_of(2) {
                TOKEN_PROGRAM_ID.to_string()
            } else {
                TOKEN_2022_PROGRAM_ID.to_string()
            },
            user_ata: addr(FIRST_ROUTE_ATA + u64::try_from(index).expect("small index")),
        })
        .collect();
    CaseParams {
        base_ata: addr(BASE_ATA),
        base_mint: header.base_mint.mint().to_string(),
        fee_wallet: addr(FEE_WALLET),
        flags: header.flags,
        max_walk_steps: header.max_walk_steps,
        menu,
        min_profit_base_units: header.min_profit_base_units.to_string(),
        route_mints,
        user: addr(USER),
    }
}

fn flags(byte: u8) -> FlagsJson {
    FlagsJson {
        flashloan: byte & 1 != 0,
        fail_if_no_profit: byte & 2 != 0,
    }
}

/// One window per point that changes a window, under a fixed header, so a red case names one
/// venue variant rather than a header interaction.
fn window_cases() -> Vec<(String, CaseParams)> {
    let mut cases: Vec<(String, CaseParams)> = Vec::new();
    let mut push = |id: String, window: WindowInput| {
        cases.push((id, params(&PLAIN, vec![window])));
    };

    push("window/RaydiumAmmV4/9".into(), amm_v4(&mut Keys::new()));
    for supplemental in 0..=3usize {
        push(
            format!("window/Whirlpool/{}", 16 + supplemental),
            whirlpool_window(&mut Keys::new(), supplemental),
        );
    }
    for tail in 1..=7usize {
        push(
            format!("window/RaydiumClmm/{}", 14 + tail),
            clmm(&mut Keys::new(), tail),
        );
    }
    push("window/RaydiumCpmm/14".into(), cpmm(&mut Keys::new()));
    for bins in 1..=8usize {
        let bitmap = !bins.is_multiple_of(2);
        let host_fee = bins >= 5;
        let tags = match (bitmap, host_fee) {
            (false, false) => String::new(),
            (true, false) => "/bitmap".into(),
            (false, true) => "/hostfee".into(),
            (true, true) => "/bitmap/hostfee".into(),
        };
        push(
            format!("window/MeteoraDlmmSwap/{}{tags}", 16 + bins),
            dlmm_swap(&mut Keys::new(), bins, bitmap, host_fee),
        );
        push(
            format!("window/MeteoraDlmmSwap2/{}{tags}", 17 + bins),
            dlmm_swap2(&mut Keys::new(), bins, bitmap, host_fee),
        );
    }
    for (form, len) in [
        (DammV2FormJson::Base, 15),
        (DammV2FormJson::RateLimited, 16),
    ] {
        push(
            format!("window/MeteoraDammV2/{len}"),
            damm_v2(&mut Keys::new(), form, false),
        );
        push(
            format!("window/MeteoraDammV2/{len}/referral"),
            damm_v2(&mut Keys::new(), form, true),
        );
    }
    push(
        "window/PumpSwapSell/24".into(),
        pump_sell(&mut Keys::new(), false, false),
    );
    push(
        "window/PumpSwapSell/25/poolv2".into(),
        pump_sell(&mut Keys::new(), false, true),
    );
    push(
        "window/PumpSwapSell/26/cashback".into(),
        pump_sell(&mut Keys::new(), true, false),
    );
    push(
        "window/PumpSwapSell/27".into(),
        pump_sell(&mut Keys::new(), true, true),
    );
    push(
        "window/PumpSwapBuy/26".into(),
        pump_buy(&mut Keys::new(), false, false),
    );
    push(
        "window/PumpSwapBuy/27/cashback".into(),
        pump_buy(&mut Keys::new(), true, false),
    );
    push(
        "window/PumpSwapBuy/27/poolv2".into(),
        pump_buy(&mut Keys::new(), false, true),
    );
    push(
        "window/PumpSwapBuy/28".into(),
        pump_buy(&mut Keys::new(), true, true),
    );
    push("window/MeteoraDammV1/16".into(), damm_v1(&mut Keys::new()));
    cases
}

fn amm_v4_menu(count: usize) -> Vec<WindowInput> {
    let mut keys = Keys::new();
    (0..count).map(|_| amm_v4(&mut keys)).collect()
}

fn header_cases() -> Vec<(String, CaseParams)> {
    let mut cases = Vec::new();
    for byte in 0..=3u8 {
        let header = Header {
            flags: flags(byte),
            ..PLAIN
        };
        cases.push((format!("flags/{byte}"), params(&header, amm_v4_menu(1))));
    }
    for mints in 1..=4usize {
        let header = Header { mints, ..PLAIN };
        cases.push((
            format!("route_mints/{mints}"),
            params(&header, amm_v4_menu(1)),
        ));
    }
    for (label, value) in [
        ("0", 0u64),
        ("1", 1),
        ("max", u64::MAX),
        ("0102030405060708", 0x0102_0304_0506_0708),
    ] {
        let header = Header {
            min_profit_base_units: value,
            ..PLAIN
        };
        cases.push((
            format!("min_profit/{label}"),
            params(&header, amm_v4_menu(1)),
        ));
    }
    for steps in [0u8, 4, 255] {
        let header = Header {
            max_walk_steps: steps,
            ..PLAIN
        };
        cases.push((
            format!("max_walk_steps/{steps}"),
            params(&header, amm_v4_menu(1)),
        ));
    }
    for base_mint in BaseMint::ALL {
        let header = Header { base_mint, ..PLAIN };
        cases.push((
            format!("base_mint/{base_mint:?}"),
            params(&header, amm_v4_menu(1)),
        ));
    }
    cases
}

fn menu_cases() -> Vec<(String, CaseParams)> {
    let mut cases = Vec::new();
    for pools in 1..=7usize {
        cases.push((
            format!("menu/pools/{pools}"),
            params(&PLAIN, amm_v4_menu(pools)),
        ));
    }

    let mut keys = Keys::new();
    let small = vec![
        cpmm(&mut keys),
        cpmm(&mut keys),
        cpmm(&mut keys),
        amm_v4(&mut keys),
        amm_v4(&mut keys),
        amm_v4(&mut keys),
    ];
    cases.push(("budget/69/small".into(), params(&PLAIN, small)));

    let mut keys = Keys::new();
    let large = vec![
        pump_buy(&mut keys, true, true),
        dlmm_swap2(&mut keys, 8, true, true),
        clmm(&mut keys, 2),
    ];
    cases.push(("budget/69/large".into(), params(&PLAIN, large)));

    let full = Header {
        mints: 4,
        flags: flags(3),
        max_walk_steps: 4,
        min_profit_base_units: 1,
        base_mint: BaseMint::Usdc,
    };
    let mut keys = Keys::new();
    let mixed_a = vec![
        amm_v4(&mut keys),
        cpmm(&mut keys),
        damm_v2(&mut keys, DammV2FormJson::Base, true),
        clmm(&mut keys, 1),
        damm_v1(&mut keys),
    ];
    cases.push(("mixed/a".into(), params(&full, mixed_a)));
    let mut keys = Keys::new();
    let mixed_b = vec![
        whirlpool_window(&mut keys, 3),
        dlmm_swap(&mut keys, 8, false, true),
        pump_buy(&mut keys, false, false),
    ];
    cases.push(("mixed/b".into(), params(&full, mixed_b)));
    let mut keys = Keys::new();
    let mixed_c = vec![
        dlmm_swap2(&mut keys, 8, true, false),
        pump_sell(&mut keys, true, true),
        clmm(&mut keys, 3),
    ];
    cases.push(("mixed/c".into(), params(&full, mixed_c)));
    cases
}

fn negative_cases() -> Vec<(String, CaseParams)> {
    let mut cases = Vec::new();
    let no_mints = Header { mints: 0, ..PLAIN };
    cases.push((
        "negative/no_route_mints".into(),
        params(&no_mints, amm_v4_menu(1)),
    ));
    let five_mints = Header { mints: 5, ..PLAIN };
    cases.push((
        "negative/too_many_route_mints/5".into(),
        params(&five_mints, amm_v4_menu(1)),
    ));
    cases.push(("negative/empty_menu".into(), params(&PLAIN, Vec::new())));
    cases.push((
        "negative/too_many_menu_pools/9".into(),
        params(&PLAIN, amm_v4_menu(9)),
    ));

    let mut keys = Keys::new();
    let five_cpmm = (0..5).map(|_| cpmm(&mut keys)).collect();
    cases.push(("negative/budget/70".into(), params(&PLAIN, five_cpmm)));

    let mut keys = Keys::new();
    let mixed_over = vec![
        amm_v4(&mut keys),
        cpmm(&mut keys),
        damm_v2(&mut keys, DammV2FormJson::RateLimited, true),
        clmm(&mut keys, 1),
        damm_v1(&mut keys),
    ];
    cases.push((
        "negative/budget/70/mixed".into(),
        params(&PLAIN, mixed_over),
    ));
    cases.push(("negative/budget/72".into(), params(&PLAIN, amm_v4_menu(8))));
    cases.push((
        "negative/order/mints_before_pools".into(),
        params(&no_mints, amm_v4_menu(9)),
    ));

    for tail in [0usize, 8] {
        cases.push((
            format!("tail/RaydiumClmm/{tail}"),
            params(&PLAIN, vec![clmm(&mut Keys::new(), tail)]),
        ));
    }
    for bins in [0usize, 9] {
        cases.push((
            format!("tail/MeteoraDlmmSwap/{bins}"),
            params(
                &PLAIN,
                vec![dlmm_swap(&mut Keys::new(), bins, false, false)],
            ),
        ));
        cases.push((
            format!("tail/MeteoraDlmmSwap2/{bins}"),
            params(
                &PLAIN,
                vec![dlmm_swap2(&mut Keys::new(), bins, false, false)],
            ),
        ));
    }
    cases
}

pub fn generate() -> Corpus {
    let mut cases = BTreeMap::new();
    for (id, params) in window_cases()
        .into_iter()
        .chain(header_cases())
        .chain(menu_cases())
        .chain(negative_cases())
    {
        let case = match build_case(&params) {
            Ok(expected) => Case {
                params,
                expected: Some(expected),
                error: None,
            },
            Err(error) => Case {
                params,
                expected: None,
                error: Some(error_json(&error)),
            },
        };
        assert!(
            cases.insert(id.clone(), case).is_none(),
            "duplicate case {id}"
        );
    }
    Corpus {
        cases,
        generator: GENERATOR.into(),
        regenerate: REGENERATE.into(),
        wire_epoch: turk_router::WIRE_EPOCH,
    }
}

/// Sorted keys, two-space indent, one trailing newline: the form `serde_json` gives a `Value`.
pub fn render(corpus: &Corpus) -> String {
    let value = serde_json::to_value(corpus).expect("the corpus serializes");
    let mut text = serde_json::to_string_pretty(&value).expect("a Value renders");
    text.push('\n');
    text
}

/// The committed corpus, parsed. Under the write switch the file may not exist yet or may be
/// mid-write by the generating test, so the readers parse the freshly rendered text instead —
/// which is the same bytes that test writes, and proves the round trip.
pub fn read_committed() -> Corpus {
    if std::env::var_os("TURK_ROUTER_WRITE_GOLDEN").is_some() {
        return serde_json::from_str(&render(&generate())).expect("the rendered corpus parses");
    }
    let path = golden_path();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("{}: {error}\nregenerate with: {REGENERATE}", path.display())
    });
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

// ---------------------------------------------------------------------------------------------
// From typed inputs to the instruction: the path the TypeScript client mirrors
// ---------------------------------------------------------------------------------------------

pub fn build_case(params: &CaseParams) -> Result<Expected, Error> {
    let menu: Vec<VenueWindow> = params
        .menu
        .iter()
        .map(window_from_input)
        .collect::<Result<_, _>>()?;
    let route_mints: Vec<RouteMint> = params
        .route_mints
        .iter()
        .map(|mint| RouteMint {
            token_program: pubkey(&mint.token_program),
            user_ata: pubkey(&mint.user_ata),
        })
        .collect();
    let instruction = build_find_route_instruction(&FindRouteParams {
        user: pubkey(&params.user),
        base_mint: base_mint(&params.base_mint),
        base_ata: pubkey(&params.base_ata),
        fee_wallet: pubkey(&params.fee_wallet),
        flags: FindRouteFlags {
            flashloan: params.flags.flashloan,
            fail_if_no_profit: params.flags.fail_if_no_profit,
        },
        max_walk_steps: params.max_walk_steps,
        min_profit_base_units: u64::from_str(&params.min_profit_base_units)
            .unwrap_or_else(|error| panic!("{}: {error}", params.min_profit_base_units)),
        route_mints: &route_mints,
        menu: &menu,
    })?;
    Ok(expected(&instruction))
}

pub fn window_from_input(input: &WindowInput) -> Result<VenueWindow, Error> {
    Ok(match input {
        WindowInput::RaydiumAmmV4 { accounts } => raydium_amm_v4::resolve(RaydiumAmmV4Accounts {
            pool: pubkey(&accounts.pool),
            base_vault: pubkey(&accounts.base_vault),
            quote_vault: pubkey(&accounts.quote_vault),
            user_source: pubkey(&accounts.user_source),
            user_destination: pubkey(&accounts.user_destination),
            payer: pubkey(&accounts.payer),
        }),
        WindowInput::Whirlpool {
            accounts,
            supplemental_tick_arrays,
        } => whirlpool::resolve(
            &WhirlpoolAccounts {
                token_program_a: pubkey(&accounts.token_program_a),
                token_program_b: pubkey(&accounts.token_program_b),
                token_authority: pubkey(&accounts.token_authority),
                whirlpool: pubkey(&accounts.whirlpool),
                mint_a: pubkey(&accounts.mint_a),
                mint_b: pubkey(&accounts.mint_b),
                token_owner_account_a: pubkey(&accounts.token_owner_account_a),
                token_vault_a: pubkey(&accounts.token_vault_a),
                token_owner_account_b: pubkey(&accounts.token_owner_account_b),
                token_vault_b: pubkey(&accounts.token_vault_b),
                tick_array_0: pubkey(&accounts.tick_array_0),
                tick_array_1: pubkey(&accounts.tick_array_1),
                tick_array_2: pubkey(&accounts.tick_array_2),
                oracle: pubkey(&accounts.oracle),
            },
            supplemental(&pubkeys(supplemental_tick_arrays)),
        ),
        WindowInput::RaydiumClmm { accounts, tail } => raydium_clmm::resolve(
            RaydiumClmmAccounts {
                payer: pubkey(&accounts.payer),
                amm_config: pubkey(&accounts.amm_config),
                pool: pubkey(&accounts.pool),
                input_token_account: pubkey(&accounts.input_token_account),
                output_token_account: pubkey(&accounts.output_token_account),
                input_vault: pubkey(&accounts.input_vault),
                output_vault: pubkey(&accounts.output_vault),
                observation_state: pubkey(&accounts.observation_state),
                input_mint: pubkey(&accounts.input_mint),
                output_mint: pubkey(&accounts.output_mint),
            },
            ClmmTail::try_from_slice(&pubkeys(tail))?,
        ),
        WindowInput::RaydiumCpmm { accounts } => raydium_cpmm::resolve(RaydiumCpmmAccounts {
            user: pubkey(&accounts.user),
            amm_config: pubkey(&accounts.amm_config),
            pool: pubkey(&accounts.pool),
            input_token_account: pubkey(&accounts.input_token_account),
            output_token_account: pubkey(&accounts.output_token_account),
            input_vault: pubkey(&accounts.input_vault),
            output_vault: pubkey(&accounts.output_vault),
            input_token_program: pubkey(&accounts.input_token_program),
            output_token_program: pubkey(&accounts.output_token_program),
            input_mint: pubkey(&accounts.input_mint),
            output_mint: pubkey(&accounts.output_mint),
            observation_state: pubkey(&accounts.observation_state),
        }),
        WindowInput::MeteoraDlmmSwap {
            accounts,
            bin_arrays,
        } => meteora_dlmm_swap::resolve(
            MeteoraDlmmSwapAccounts {
                lb_pair: pubkey(&accounts.lb_pair),
                bin_array_bitmap_extension: optional(&accounts.bin_array_bitmap_extension),
                reserve_x: pubkey(&accounts.reserve_x),
                reserve_y: pubkey(&accounts.reserve_y),
                user_token_in: pubkey(&accounts.user_token_in),
                user_token_out: pubkey(&accounts.user_token_out),
                mint_x: pubkey(&accounts.mint_x),
                mint_y: pubkey(&accounts.mint_y),
                oracle: pubkey(&accounts.oracle),
                host_fee_in: optional(&accounts.host_fee_in),
                user: pubkey(&accounts.user),
            },
            BinArrayTail::try_from_slice(&pubkeys(bin_arrays))?,
        ),
        WindowInput::MeteoraDlmmSwap2 {
            accounts,
            bin_arrays,
        } => meteora_dlmm_swap2::resolve(
            MeteoraDlmmSwap2Accounts {
                pool: pubkey(&accounts.pool),
                bin_array_bitmap_extension: optional(&accounts.bin_array_bitmap_extension),
                reserve_x: pubkey(&accounts.reserve_x),
                reserve_y: pubkey(&accounts.reserve_y),
                user_token_in: pubkey(&accounts.user_token_in),
                user_token_out: pubkey(&accounts.user_token_out),
                token_x_mint: pubkey(&accounts.token_x_mint),
                token_y_mint: pubkey(&accounts.token_y_mint),
                oracle: pubkey(&accounts.oracle),
                host_fee_in: optional(&accounts.host_fee_in),
                user: pubkey(&accounts.user),
                token_x_program: pubkey(&accounts.token_x_program),
                token_y_program: pubkey(&accounts.token_y_program),
            },
            PubkeyTail::<MAX_BIN_ARRAYS>::try_from_slice(&pubkeys(bin_arrays))?,
        ),
        WindowInput::MeteoraDammV2 { accounts, form } => meteora_damm_v2::resolve(
            MeteoraDammV2Accounts {
                pool: pubkey(&accounts.pool),
                input_token_account: pubkey(&accounts.input_token_account),
                output_token_account: pubkey(&accounts.output_token_account),
                token_a_vault: pubkey(&accounts.token_a_vault),
                token_b_vault: pubkey(&accounts.token_b_vault),
                token_a_mint: pubkey(&accounts.token_a_mint),
                token_b_mint: pubkey(&accounts.token_b_mint),
                payer: pubkey(&accounts.payer),
                token_a_program: pubkey(&accounts.token_a_program),
                token_b_program: pubkey(&accounts.token_b_program),
                referral_token_account: optional(&accounts.referral_token_account),
            },
            match form {
                DammV2FormJson::Base => DammV2Form::Base,
                DammV2FormJson::RateLimited => DammV2Form::RateLimited,
            },
        ),
        WindowInput::PumpSwapSell { accounts } => pump_swap_sell::resolve(PumpSwapSellAccounts {
            pool: pubkey(&accounts.pool),
            user: pubkey(&accounts.user),
            forwarded_before_base_mint: pubkey(&accounts.forwarded_before_base_mint),
            base_mint: pubkey(&accounts.base_mint),
            quote_mint: pubkey(&accounts.quote_mint),
            base_ata: pubkey(&accounts.base_ata),
            quote_ata: pubkey(&accounts.quote_ata),
            base_vault: pubkey(&accounts.base_vault),
            quote_vault: pubkey(&accounts.quote_vault),
            forwarded_before_fee_config: ten(&accounts.forwarded_before_fee_config),
            cashback: accounts.cashback.as_ref().map(two),
            pool_v2: optional(&accounts.pool_v2),
            forwarded_close: two(&accounts.forwarded_close),
        }),
        WindowInput::PumpSwapBuy { accounts } => pump_swap_buy::resolve(PumpSwapBuyAccounts {
            pool: pubkey(&accounts.pool),
            user: pubkey(&accounts.user),
            forwarded_before_base_mint: pubkey(&accounts.forwarded_before_base_mint),
            base_mint: pubkey(&accounts.base_mint),
            quote_mint: pubkey(&accounts.quote_mint),
            base_token_account: pubkey(&accounts.base_token_account),
            quote_token_account: pubkey(&accounts.quote_token_account),
            base_vault: pubkey(&accounts.base_vault),
            quote_vault: pubkey(&accounts.quote_vault),
            forwarded_before_volume_accumulator: ten(&accounts.forwarded_before_volume_accumulator),
            user_volume_accumulator: pubkey(&accounts.user_volume_accumulator),
            forwarded_close: two(&accounts.forwarded_close),
            pool_v2: optional(&accounts.pool_v2),
            cashback: optional(&accounts.cashback),
        }),
        WindowInput::MeteoraDammV1 { accounts } => {
            meteora_damm_v1::resolve(MeteoraDammV1Accounts {
                pool: pubkey(&accounts.pool),
                user_source: pubkey(&accounts.user_source),
                user_dest: pubkey(&accounts.user_dest),
                a_vault: pubkey(&accounts.a_vault),
                b_vault: pubkey(&accounts.b_vault),
                a_token_vault: pubkey(&accounts.a_token_vault),
                b_token_vault: pubkey(&accounts.b_token_vault),
                a_vault_lp_mint: pubkey(&accounts.a_vault_lp_mint),
                b_vault_lp_mint: pubkey(&accounts.b_vault_lp_mint),
                a_vault_lp: pubkey(&accounts.a_vault_lp),
                b_vault_lp: pubkey(&accounts.b_vault_lp),
                protocol_token_fee: pubkey(&accounts.protocol_token_fee),
                payer: pubkey(&accounts.payer),
            })
        }
    })
}

/// The fourth array is a type error in both clients, so the corpus never carries one; a file
/// that does is a broken file, not a case.
fn supplemental(keys: &[Pubkey]) -> SupplementalTickArrays {
    match keys {
        [] => SupplementalTickArrays::from([]),
        [a] => SupplementalTickArrays::from([*a]),
        [a, b] => SupplementalTickArrays::from([*a, *b]),
        [a, b, c] => SupplementalTickArrays::from([*a, *b, *c]),
        other => panic!(
            "{} supplemental tick arrays: the corpus carries at most three",
            other.len()
        ),
    }
}

pub fn pubkey(text: &str) -> Pubkey {
    Pubkey::from_str(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn optional(text: &Option<Addr>) -> Option<Pubkey> {
    text.as_deref().map(pubkey)
}

fn pubkeys(texts: &[Addr]) -> Vec<Pubkey> {
    texts.iter().map(|text| pubkey(text)).collect()
}

fn ten(texts: &[Addr; 10]) -> [Pubkey; 10] {
    std::array::from_fn(|index| pubkey(&texts[index]))
}

fn two(texts: &[Addr; 2]) -> [Pubkey; 2] {
    [pubkey(&texts[0]), pubkey(&texts[1])]
}

pub fn base_mint(text: &str) -> BaseMint {
    let mint = pubkey(text);
    BaseMint::ALL
        .into_iter()
        .find(|candidate| candidate.mint() == mint)
        .unwrap_or_else(|| panic!("{text} is not a base mint"))
}

pub fn expected(instruction: &Instruction) -> Expected {
    Expected {
        accounts: instruction.accounts.iter().map(meta_text).collect(),
        data_hex: instruction
            .data
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        program_address: instruction.program_id.to_string(),
    }
}

/// `<address>:<role>`, with Kit's `AccountRole` names; base58 never contains a colon.
pub fn meta_text(meta: &AccountMeta) -> String {
    let role = match (meta.is_signer, meta.is_writable) {
        (false, false) => "readonly",
        (false, true) => "writable",
        (true, false) => "readonly_signer",
        (true, true) => "writable_signer",
    };
    format!("{}:{role}", meta.pubkey)
}

/// `{ kind, ...fields }` with the variant's own field names. `Error` is `#[non_exhaustive]`, so
/// the wildcard arm is what the compiler demands; a variant the corpus cannot spell is a panic,
/// not a silent omission.
pub fn error_json(error: &Error) -> Value {
    match error {
        Error::UnknownHopKind { raw } => json!({ "kind": "UnknownHopKind", "raw": raw }),
        Error::NoRouteMints => json!({ "kind": "NoRouteMints" }),
        Error::TooManyRouteMints { given, max } => {
            json!({ "kind": "TooManyRouteMints", "given": given, "max": max })
        }
        Error::EmptyMenu => json!({ "kind": "EmptyMenu" }),
        Error::TooManyMenuPools { given, max } => {
            json!({ "kind": "TooManyMenuPools", "given": given, "max": max })
        }
        Error::MenuAccountBudgetExceeded { declared, budget } => {
            json!({ "kind": "MenuAccountBudgetExceeded", "declared": declared, "budget": budget })
        }
        Error::TailLength { given, min, max } => {
            json!({ "kind": "TailLength", "given": given, "min": min, "max": max })
        }
        other => panic!("no corpus form for {other:?}"),
    }
}
