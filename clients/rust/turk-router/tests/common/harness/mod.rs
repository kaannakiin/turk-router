//! Feeds a fixture's slots into the venue module it belongs to and returns the window the module
//! builds. One function per kind; the caller compares the result to the fixture slot by slot.

use turk_router::{HopKind, VenueWindow};

use super::fixture::Fixture;

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

pub fn resolve(fixture: &Fixture) -> VenueWindow {
    let kind = HopKind::try_from(fixture.hop_kind)
        .unwrap_or_else(|error| panic!("{}: {error}", fixture.pool_b58));
    match kind {
        HopKind::RaydiumAmmV4 => raydium_amm_v4::resolve(fixture),
        HopKind::Whirlpool => whirlpool::resolve(fixture),
        HopKind::RaydiumClmm => raydium_clmm::resolve(fixture),
        HopKind::RaydiumCpmm => raydium_cpmm::resolve(fixture),
        HopKind::MeteoraDlmmSwap => meteora_dlmm_swap::resolve(fixture),
        HopKind::MeteoraDlmmSwap2 => meteora_dlmm_swap2::resolve(fixture),
        HopKind::MeteoraDammV2 => meteora_damm_v2::resolve(fixture),
        HopKind::PumpSwapSell => pump_swap_sell::resolve(fixture),
        HopKind::PumpSwapBuy => pump_swap_buy::resolve(fixture),
        HopKind::MeteoraDammV1 => meteora_damm_v1::resolve(fixture),
    }
}

/// Every account count the kind's module can declare, one window per point of its parameter
/// space, built from placeholder addresses.
pub fn reachable_account_counts(kind: HopKind) -> Vec<u8> {
    match kind {
        HopKind::RaydiumAmmV4 => raydium_amm_v4::reachable_account_counts(),
        HopKind::Whirlpool => whirlpool::reachable_account_counts(),
        HopKind::RaydiumClmm => raydium_clmm::reachable_account_counts(),
        HopKind::RaydiumCpmm => raydium_cpmm::reachable_account_counts(),
        HopKind::MeteoraDlmmSwap => meteora_dlmm_swap::reachable_account_counts(),
        HopKind::MeteoraDlmmSwap2 => meteora_dlmm_swap2::reachable_account_counts(),
        HopKind::MeteoraDammV2 => meteora_damm_v2::reachable_account_counts(),
        HopKind::PumpSwapSell => pump_swap_sell::reachable_account_counts(),
        HopKind::PumpSwapBuy => pump_swap_buy::reachable_account_counts(),
        HopKind::MeteoraDammV1 => meteora_damm_v1::reachable_account_counts(),
    }
}
